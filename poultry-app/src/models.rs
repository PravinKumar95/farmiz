use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EggSale {
    pub id: String,
    pub date: String,
    pub party_name: String,
    pub quantity_boxes: i32,
    pub total_eggs: i32,
    pub size: String,
    pub gross_rate: f64,
    pub less_discount: f64,
    pub net_rate: f64,
    pub total_amount: f64,
    pub received_amount: f64,
    pub payment_mode: String,
    pub balance: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokenEggSale {
    pub id: String,
    pub date: String,
    pub bakery_name: String,
    pub trays_sold: i32,
    pub rate: f64,
    pub amount: f64,
    pub payment_received: f64,
    pub return_trays: i32,
    pub empty_trays_balance: i32,
    pub balance_amount: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialPurchase {
    pub id: String,
    pub date: String,
    pub material_name: String,
    pub party_name: String,
    pub quantity_kg: f64,
    pub rate_per_kg: f64,
    pub total_amount: f64,
    pub advance_paid: f64,
    pub status: String,
    pub balance: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedBatch {
    pub id: String,
    pub date: String,
    pub batch_id: String,
    pub feed_type: String,
    pub rate: f64,
    pub total_amount: f64,
    pub payment: f64,
    pub opening_balance: f64,
    pub closing_balance: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaborRecord {
    pub id: String,
    pub date: String,
    pub employee_name: String,
    pub attendance: f64,
    pub advance_given: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub party_type: String,
    pub current_balance: f64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardStats {
    pub today_sales: f64,
    pub eggs_sold_today: i32,
    pub today_purchases: f64,
    pub active_parties: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    pub date: String,
    pub description: String,
    pub charge: f64,
    pub payment: f64,
    pub created_at: Option<String>,
}
