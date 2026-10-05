use dioxus::prelude::*;
use dioxus_sdk::storage::{use_storage, LocalStorage};
use crate::models::{
    BrokenEggSale, DailyProduction, EggSale, Employee, FeedBatch, LaborRecord, MaterialPurchase, Party,
};

pub const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:9000/lambda-url/poultry-backend",
};

pub fn parse_jwt_exp(token: &str) -> Option<i64> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;

    use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
    use base64::Engine;

    let decoded = URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .or_else(|_| URL_SAFE.decode(payload.as_bytes()))
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("exp").and_then(|e| e.as_i64())
}

pub fn is_jwt_expired(token: &str) -> bool {
    if token.trim().is_empty() {
        return true;
    }
    if let Some(exp) = parse_jwt_exp(token) {
        let now = chrono::Utc::now().timestamp();
        // Expired if current timestamp is at or past (exp - 10s)
        now >= (exp - 10)
    } else {
        false
    }
}

#[derive(Clone, Copy)]
pub struct AuthSession {
    pub token: Signal<Option<String>>,
    pub session_cookies: Signal<Option<Vec<String>>>,
    pub auth_email: Signal<Option<String>>,
    pub session_expired: Signal<bool>,
}

pub fn use_auth() -> AuthSession {
    if let Some(auth) = try_use_context::<AuthSession>() {
        auth
    } else {
        let token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
        let auth_email = use_storage::<LocalStorage, _>("auth_email".to_string(), || None::<String>);
        let session_cookies = use_storage::<LocalStorage, _>("session_cookies".to_string(), || None::<Vec<String>>);
        let session_expired = use_signal(|| false);
        AuthSession {
            token,
            session_cookies,
            auth_email,
            session_expired,
        }
    }
}

impl AuthSession {
    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.token.read().is_some()
    }

    pub fn logout(&self) {
        let mut token = self.token;
        let mut session_cookies = self.session_cookies;
        let mut auth_email = self.auth_email;
        let mut session_expired = self.session_expired;
        token.set(None);
        session_cookies.set(None);
        auth_email.set(None);
        session_expired.set(false);
    }

    pub fn expire_session(&self) {
        let mut token = self.token;
        let mut session_cookies = self.session_cookies;
        let mut auth_email = self.auth_email;
        let mut session_expired = self.session_expired;
        token.set(None);
        session_cookies.set(None);
        auth_email.set(None);
        session_expired.set(true);
    }

    pub async fn refresh(&self) -> Result<String, String> {
        let cookies = match self.session_cookies.read().clone() {
            Some(c) if !c.is_empty() => c,
            _ => {
                self.expire_session();
                return Err("No session cookies".into());
            }
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("{BACKEND_URL}/api/auth/refresh"))
            .json(&serde_json::json!({ "session_cookies": cookies }))
            .send()
            .await
            .map_err(|e| {
                self.expire_session();
                e.to_string()
            })?;

        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|_| {
                self.expire_session();
                "Invalid JSON".to_string()
            })?;
            let token = json
                .get("token")
                .and_then(|t| t.as_str())
                .ok_or_else(|| {
                    self.expire_session();
                    "Missing token".to_string()
                })?;

            let mut auth_token = self.token;
            auth_token.set(Some(token.to_string()));

            if let Some(new_cookies) = json.get("session_cookies").and_then(|c| {
                c.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
            }) {
                let mut session_cookies = self.session_cookies;
                session_cookies.set(Some(new_cookies));
            }

            Ok(token.to_string())
        } else {
            let err_msg = res
                .text()
                .await
                .unwrap_or_else(|_| "Refresh failed".to_string());
            self.expire_session();
            Err(err_msg)
        }
    }

    pub async fn ensure_valid_token(&self) -> Result<String, String> {
        let current_token = self.token.read().clone().unwrap_or_default();
        if current_token.is_empty() || is_jwt_expired(&current_token) {
            self.refresh().await
        } else {
            Ok(current_token)
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, String> {
        let client = reqwest::Client::new();
        let mut current_token = self.ensure_valid_token().await?;

        let mut res = client
            .get(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status() == 401 {
            match self.refresh().await {
                Ok(t) => {
                    current_token = t;
                    res = client
                        .get(format!("{BACKEND_URL}{endpoint}"))
                        .header("Authorization", format!("Bearer {}", current_token))
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    if res.status() == 401 {
                        self.expire_session();
                        return Err("Session expired. Please sign in again.".to_string());
                    }
                }
                Err(_) => {
                    self.expire_session();
                    return Err("Session expired. Please sign in again.".to_string());
                }
            }
        }

        if res.status().is_success() {
            res.json::<T>().await.map_err(|e| e.to_string())
        } else {
            Err(res.text().await.unwrap_or_else(|_| "API Error".to_string()))
        }
    }

    pub async fn post<T: serde::Serialize>(&self, endpoint: &str, payload: &T) -> Result<(), String> {
        let client = reqwest::Client::new();
        let mut current_token = self.ensure_valid_token().await?;

        let mut res = client
            .post(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status() == 401 {
            match self.refresh().await {
                Ok(t) => {
                    current_token = t;
                    res = client
                        .post(format!("{BACKEND_URL}{endpoint}"))
                        .header("Authorization", format!("Bearer {}", current_token))
                        .json(payload)
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    if res.status() == 401 {
                        self.expire_session();
                        return Err("Session expired. Please sign in again.".to_string());
                    }
                }
                Err(_) => {
                    self.expire_session();
                    return Err("Session expired. Please sign in again.".to_string());
                }
            }
        }

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_else(|_| "API Error".to_string()))
        }
    }

    pub async fn put<T: serde::Serialize>(&self, endpoint: &str, payload: &T) -> Result<(), String> {
        let client = reqwest::Client::new();
        let mut current_token = self.ensure_valid_token().await?;

        let mut res = client
            .put(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .json(payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status() == 401 {
            match self.refresh().await {
                Ok(t) => {
                    current_token = t;
                    res = client
                        .put(format!("{BACKEND_URL}{endpoint}"))
                        .header("Authorization", format!("Bearer {}", current_token))
                        .json(payload)
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    if res.status() == 401 {
                        self.expire_session();
                        return Err("Session expired. Please sign in again.".to_string());
                    }
                }
                Err(_) => {
                    self.expire_session();
                    return Err("Session expired. Please sign in again.".to_string());
                }
            }
        }

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_else(|_| "API Error".to_string()))
        }
    }

    pub async fn delete(&self, endpoint: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        let mut current_token = self.ensure_valid_token().await?;

        let mut res = client
            .delete(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status() == 401 {
            match self.refresh().await {
                Ok(t) => {
                    current_token = t;
                    res = client
                        .delete(format!("{BACKEND_URL}{endpoint}"))
                        .header("Authorization", format!("Bearer {}", current_token))
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    if res.status() == 401 {
                        self.expire_session();
                        return Err("Session expired. Please sign in again.".to_string());
                    }
                }
                Err(_) => {
                    self.expire_session();
                    return Err("Session expired. Please sign in again.".to_string());
                }
            }
        }

        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.text().await.unwrap_or_else(|_| "API Error".to_string()))
        }
    }
}

// Helpers for the `use_resource` hooks (since they run in components, they just use AuthSession)

pub fn use_egg_sales() -> Resource<Result<Vec<EggSale>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<EggSale>>("/api/sales/egg").await
        }
    })
}

pub fn use_broken_egg_sales() -> Resource<Result<Vec<BrokenEggSale>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<BrokenEggSale>>("/api/sales/broken").await
        }
    })
}

pub fn use_material_purchases() -> Resource<Result<Vec<MaterialPurchase>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<MaterialPurchase>>("/api/purchases").await
        }
    })
}

pub fn use_feed_batches() -> Resource<Result<Vec<FeedBatch>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<FeedBatch>>("/api/feed").await
        }
    })
}

pub fn use_labor_records() -> Resource<Result<Vec<LaborRecord>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<LaborRecord>>("/api/labor").await
        }
    })
}

pub fn use_parties() -> Resource<Result<Vec<Party>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<Party>>("/api/parties").await
        }
    })
}

pub fn use_employees() -> Resource<Result<Vec<Employee>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<Employee>>("/api/employees").await
        }
    })
}

pub fn use_dashboard_stats() -> Resource<Result<crate::models::DashboardStats, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<crate::models::DashboardStats>("/api/dashboard/stats").await
        }
    })
}

pub fn use_party_ledger(party_id: String) -> Resource<Result<Vec<crate::models::LedgerEntry>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        let id = party_id.clone();
        async move {
            auth.get::<Vec<crate::models::LedgerEntry>>(&format!("/api/parties/{}/ledger", id)).await
        }
    })
}

pub fn use_daily_production() -> Resource<Result<Vec<DailyProduction>, String>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<DailyProduction>>("/api/production").await
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    fn create_test_jwt(exp: i64) -> String {
        let header = URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
        let payload_json = serde_json::json!({
            "sub": "test_user",
            "exp": exp,
        });
        let payload = URL_SAFE_NO_PAD.encode(payload_json.to_string().as_bytes());
        format!("{}.{}.dummy_signature", header, payload)
    }

    #[test]
    fn test_parse_jwt_exp_valid() {
        let exp_ts = 1893456000;
        let jwt = create_test_jwt(exp_ts);
        assert_eq!(parse_jwt_exp(&jwt), Some(exp_ts));
    }

    #[test]
    fn test_parse_jwt_exp_invalid() {
        assert_eq!(parse_jwt_exp("invalid_token"), None);
        assert_eq!(parse_jwt_exp(""), None);
        assert_eq!(parse_jwt_exp("a.b"), None);
    }

    #[test]
    fn test_is_jwt_expired() {
        assert!(is_jwt_expired(""));
        assert!(is_jwt_expired("   "));

        let past_exp = chrono::Utc::now().timestamp() - 500;
        let expired_jwt = create_test_jwt(past_exp);
        assert!(is_jwt_expired(&expired_jwt));

        let future_exp = chrono::Utc::now().timestamp() + 500;
        let valid_jwt = create_test_jwt(future_exp);
        assert!(!is_jwt_expired(&valid_jwt));

        // Grace period: within 10 seconds of expiry should be considered expired
        let near_expiry = chrono::Utc::now().timestamp() + 5;
        let near_expired_jwt = create_test_jwt(near_expiry);
        assert!(is_jwt_expired(&near_expired_jwt));
    }
}


