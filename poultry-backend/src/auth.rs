use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, StatusCode},
    Json,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    // Add other claims here if needed
}

#[derive(Debug, Deserialize)]
struct Jwk {
    pub kid: String,
    pub kty: String,
    pub alg: Option<String>,
    pub crv: Option<String>,
    pub x: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

static JWKS: OnceCell<Jwks> = OnceCell::new();

pub async fn init_jwks() {
    let jwks_url = env::var("NEON_AUTH_JWKS_URL").expect("NEON_AUTH_JWKS_URL must be set");
    let client = reqwest::Client::new();
    
    let mut retries = 5;
    let mut last_error = String::new();

    while retries > 0 {
        match client.get(&jwks_url).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    let jwks: Jwks = res.json().await.expect("Failed to parse JWKS");
                    JWKS.set(jwks).expect("JWKS already initialized");
                    return;
                } else {
                    last_error = format!("Status {}: {}", res.status(), res.text().await.unwrap_or_default());
                }
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }
        
        retries -= 1;
        if retries > 0 {
            lambda_http::tracing::warn!("JWKS fetch failed, retrying in 2 seconds... Error: {}", last_error);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    panic!("Failed to fetch JWKS after retries. Last error: {}", last_error);
}

pub struct AuthenticatedUser {
    pub user_id: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        lambda_http::tracing::info!("AuthenticatedUser extraction started");
        
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        lambda_http::tracing::info!("Extracting kid from header");

        let header = decode_header(token).map_err(|e| (StatusCode::UNAUTHORIZED, format!("decode_header failed: {} for token: {:.10}...", e, token)))?;
        let kid = header
            .kid
            .ok_or((StatusCode::UNAUTHORIZED, "Missing kid in token header".to_string()))?;

        lambda_http::tracing::info!("Fetching JWKS for kid: {}", kid);
        let jwks = JWKS
            .get()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "JWKS not initialized".to_string()))?;

        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid kid".to_string()))?;

        // Extract the base64url encoded 'x' coordinate
        let x = jwk.x.as_deref().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing 'x' in JWK".to_string()))?;

        lambda_http::tracing::info!("Creating DecodingKey from Ed components");
        let decoding_key = DecodingKey::from_ed_components(x)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_aud = false; // Disable audience validation as per user request

        lambda_http::tracing::info!("Decoding token with jsonwebtoken");
        let decoded = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("decode failed: {}", e)))?;

        lambda_http::tracing::info!("Successfully authenticated user: {}", decoded.claims.sub);
        Ok(AuthenticatedUser {
            user_id: decoded.claims.sub,
        })
    }
}

pub async fn signin(headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let neon_auth_url = env::var("NEON_AUTH_URL")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "NEON_AUTH_URL missing".to_string()))?;

    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http://localhost:8080");

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/sign-in/email", neon_auth_url))
        .header("Origin", origin.clone())
        .header("Referer", origin)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let status = res.status();
    
    // Extract the set-cookie header to get the session token
    let cookies = res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
        .collect::<Vec<_>>();
        
    let json = res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !status.is_success() {
        return Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_REQUEST),
            json.to_string(),
        ));
    }
    
    // If signin was successful, fetch the JWT token using the session cookie
    if !cookies.is_empty() {
        lambda_http::tracing::info!("Attempting to fetch JWT with {} cookies", cookies.len());
        let mut jwt_req = client.get(format!("{}/token", neon_auth_url));
        for cookie in &cookies {
            jwt_req = jwt_req.header(reqwest::header::COOKIE, cookie);
        }
        
        match jwt_req.send().await {
            Ok(jwt_res) => {
                let status = jwt_res.status();
                lambda_http::tracing::info!("JWT fetch status: {}", status);
                if status.is_success() {
                    match jwt_res.json::<serde_json::Value>().await {
                        Ok(jwt_json) => {
                            if let Some(jwt_token) = jwt_json.get("token") {
                                lambda_http::tracing::info!("Successfully extracted JWT token");
                                let mut final_json = json.clone();
                                if let Some(obj) = final_json.as_object_mut() {
                                    obj.insert("token".to_string(), jwt_token.clone());
                                }
                                return Ok(Json(final_json));
                            } else {
                                lambda_http::tracing::error!("No 'token' field in JWT response: {:?}", jwt_json);
                            }
                        }
                        Err(e) => {
                            lambda_http::tracing::error!("Failed to parse JWT response: {}", e);
                        }
                    }
                } else {
                    let text = jwt_res.text().await.unwrap_or_default();
                    lambda_http::tracing::error!("JWT fetch failed with status {}: {}", status, text);
                    return Err((
                        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        format!("Failed to fetch JWT: {}", text),
                    ));
                }
            }
            Err(e) => {
                lambda_http::tracing::error!("Failed to send JWT request: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("JWT request error: {}", e)));
            }
        }
    } else {
        lambda_http::tracing::warn!("No cookies returned from signin, cannot fetch JWT");
    }

    Ok(Json(json))
}

pub async fn signup(headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let neon_auth_url = env::var("NEON_AUTH_URL")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "NEON_AUTH_URL missing".to_string()))?;

    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http://localhost:8080");

    lambda_http::tracing::info!("Sign-up request for email: {}", payload.get("email").and_then(|e| e.as_str()).unwrap_or("unknown"));

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/sign-up/email", neon_auth_url))
        .header("Origin", origin)
        .header("Referer", origin)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let status = res.status();
    let body = res
        .text()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    lambda_http::tracing::info!("Neon Auth sign-up response status={}, body={}", status, body);

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse Neon Auth response: {} — raw: {}", e, body)))?;

    if !status.is_success() {
        return Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_REQUEST),
            json.to_string(),
        ));
    }

    Ok(Json(json))
}

pub async fn verify_email(headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let neon_auth_url = env::var("NEON_AUTH_URL")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "NEON_AUTH_URL missing".to_string()))?;

    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http://localhost:8080");

    lambda_http::tracing::info!("Verify email request for: {}", payload.get("email").and_then(|e| e.as_str()).unwrap_or("unknown"));

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/email-otp/verify-email", neon_auth_url))
        .header("Origin", origin)
        .header("Referer", origin)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let status = res.status();
    let body = res
        .text()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    lambda_http::tracing::info!("Neon Auth verify-email response status={}, body={}", status, body);

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse verify response: {} — raw: {}", e, body)))?;

    if !status.is_success() {
        return Err((
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_REQUEST),
            json.to_string(),
        ));
    }

    Ok(Json(json))
}

