use dioxus::prelude::*;
use dioxus_sdk::storage::{use_storage, LocalStorage};
use crate::models::{
    BrokenEggSale, EggSale, FeedBatch, LaborRecord, MaterialPurchase, Party,
};

const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://localhost:3000",
};

pub fn use_egg_sales() -> Resource<Vec<EggSale>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/sales/egg"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<EggSale>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_egg_sale(token: &str, payload: &EggSale) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/sales/egg"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}

pub fn use_broken_egg_sales() -> Resource<Vec<BrokenEggSale>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/sales/broken"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<BrokenEggSale>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_broken_egg_sale(token: &str, payload: &BrokenEggSale) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/sales/broken"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}

pub fn use_material_purchases() -> Resource<Vec<MaterialPurchase>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/purchases"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<MaterialPurchase>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_material_purchase(token: &str, payload: &MaterialPurchase) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/purchases"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}

pub fn use_feed_batches() -> Resource<Vec<FeedBatch>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/feed"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<FeedBatch>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_feed_batch(token: &str, payload: &FeedBatch) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/feed"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}

pub fn use_labor_records() -> Resource<Vec<LaborRecord>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/labor"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<LaborRecord>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_labor_record(token: &str, payload: &LaborRecord) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/labor"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}

pub fn use_parties() -> Resource<Vec<Party>> {
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    use_resource(move || {
        let token = auth_token.read().clone().unwrap_or_default();
        async move {
            let client = reqwest::Client::new();
            match client.get(format!("{BACKEND_URL}/api/parties"))
                .header("Authorization", format!("Bearer {}", token))
                .send().await {
                Ok(res) => res.json::<Vec<Party>>().await.unwrap_or_default(),
                Err(_) => vec![],
            }
        }
    })
}

pub async fn create_party(token: &str, payload: &Party) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client.post(format!("{BACKEND_URL}/api/parties"))
        .header("Authorization", format!("Bearer {}", token))
        .json(payload)
        .send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.text().await.unwrap_or_else(|_| "Unknown API error".to_string()))
    }
}
