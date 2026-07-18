use dioxus::prelude::*;
use dioxus_sdk::storage::{use_storage, LocalStorage};
use crate::models::{
    BrokenEggSale, EggSale, FeedBatch, LaborRecord, MaterialPurchase, Party, Employee
};

const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://localhost:3000",
};

#[derive(Clone, Copy)]
pub struct AuthSession {
    pub token: Signal<Option<String>>,
    pub session_cookies: Signal<Option<Vec<String>>>,
}

pub fn use_auth() -> AuthSession {
    let token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    let session_cookies = use_storage::<LocalStorage, _>("session_cookies".to_string(), || None::<Vec<String>>);
    AuthSession { token, session_cookies }
}

impl AuthSession {
    pub async fn refresh(&self) -> Result<String, String> {
        let cookies = self.session_cookies.read().clone().ok_or("No session cookies")?;
        if cookies.is_empty() { return Err("No session cookies".into()); }
        
        let client = reqwest::Client::new();
        let res = client.post(format!("{BACKEND_URL}/api/auth/refresh"))
            .json(&serde_json::json!({ "session_cookies": cookies }))
            .send().await.map_err(|e| e.to_string())?;
            
        if res.status().is_success() {
            let json: serde_json::Value = res.json().await.map_err(|_| "Invalid JSON")?;
            let token = json.get("token").and_then(|t| t.as_str()).ok_or("Missing token")?;
            let mut auth_token = self.token;
            auth_token.set(Some(token.to_string()));
            Ok(token.to_string())
        } else {
            Err(res.text().await.unwrap_or_else(|_| "Refresh failed".to_string()))
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, String> {
        let client = reqwest::Client::new();
        let mut current_token = self.token.read().clone().unwrap_or_default();
        
        let mut res = client.get(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .send().await.map_err(|e| e.to_string())?;
            
        if res.status() == 401 {
            if let Ok(t) = self.refresh().await {
                current_token = t;
                res = client.get(format!("{BACKEND_URL}{endpoint}"))
                    .header("Authorization", format!("Bearer {}", current_token))
                    .send().await.map_err(|e| e.to_string())?;
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
        let mut current_token = self.token.read().clone().unwrap_or_default();
        
        let mut res = client.post(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .json(payload)
            .send().await.map_err(|e| e.to_string())?;
            
        if res.status() == 401 {
            if let Ok(t) = self.refresh().await {
                current_token = t;
                res = client.post(format!("{BACKEND_URL}{endpoint}"))
                    .header("Authorization", format!("Bearer {}", current_token))
                    .json(payload)
                    .send().await.map_err(|e| e.to_string())?;
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
        let mut current_token = self.token.read().clone().unwrap_or_default();
        
        let mut res = client.put(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .json(payload)
            .send().await.map_err(|e| e.to_string())?;
            
        if res.status() == 401 {
            if let Ok(t) = self.refresh().await {
                current_token = t;
                res = client.put(format!("{BACKEND_URL}{endpoint}"))
                    .header("Authorization", format!("Bearer {}", current_token))
                    .json(payload)
                    .send().await.map_err(|e| e.to_string())?;
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
        let mut current_token = self.token.read().clone().unwrap_or_default();
        
        let mut res = client.delete(format!("{BACKEND_URL}{endpoint}"))
            .header("Authorization", format!("Bearer {}", current_token))
            .send().await.map_err(|e| e.to_string())?;
            
        if res.status() == 401 {
            if let Ok(t) = self.refresh().await {
                current_token = t;
                res = client.delete(format!("{BACKEND_URL}{endpoint}"))
                    .header("Authorization", format!("Bearer {}", current_token))
                    .send().await.map_err(|e| e.to_string())?;
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

pub fn use_egg_sales() -> Resource<Vec<EggSale>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<EggSale>>("/api/sales/egg").await.unwrap_or_default()
        }
    })
}

pub fn use_broken_egg_sales() -> Resource<Vec<BrokenEggSale>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<BrokenEggSale>>("/api/sales/broken").await.unwrap_or_default()
        }
    })
}

pub fn use_material_purchases() -> Resource<Vec<MaterialPurchase>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<MaterialPurchase>>("/api/purchases").await.unwrap_or_default()
        }
    })
}

pub fn use_feed_batches() -> Resource<Vec<FeedBatch>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<FeedBatch>>("/api/feed").await.unwrap_or_default()
        }
    })
}

pub fn use_labor_records() -> Resource<Vec<LaborRecord>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<LaborRecord>>("/api/labor").await.unwrap_or_default()
        }
    })
}

pub fn use_parties() -> Resource<Vec<Party>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<Party>>("/api/parties").await.unwrap_or_default()
        }
    })
}

pub fn use_employees() -> Resource<Vec<Employee>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<Vec<Employee>>("/api/employees").await.unwrap_or_default()
        }
    })
}

pub fn use_dashboard_stats() -> Resource<Option<crate::models::DashboardStats>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        async move {
            auth.get::<crate::models::DashboardStats>("/api/dashboard/stats").await.ok()
        }
    })
}

pub fn use_party_ledger(party_id: String) -> Resource<Vec<crate::models::LedgerEntry>> {
    let auth = use_auth();
    use_resource(move || {
        let auth = auth;
        let id = party_id.clone();
        async move {
            auth.get::<Vec<crate::models::LedgerEntry>>(&format!("/api/parties/{}/ledger", id)).await.unwrap_or_default()
        }
    })
}
