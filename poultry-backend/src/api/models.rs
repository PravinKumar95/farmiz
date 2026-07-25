use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DashboardStats {
    pub today_sales: f64,
    pub eggs_sold_today: i32,
    pub today_purchases: f64,
    pub active_parties: i64,
}

#[derive(Serialize)]
pub struct LedgerEntry {
    pub date: String,
    pub description: String,
    pub charge: f64,
    pub payment: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Party {
    pub id: Uuid,
    pub name: String,
    pub party_type: String,
    pub current_balance: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateParty {
    pub name: String,
    pub party_type: String,
    pub current_balance: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub daily_wage: f64,
    pub current_balance: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateEmployee {
    pub name: String,
    pub role: String,
    pub daily_wage: f64,
    pub current_balance: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct EggSale {
    pub id: Uuid,
    pub date: String,
    pub party_name: String,
    pub party_id: Option<Uuid>,
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
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateEggSale {
    pub date: String,
    pub party_name: String,
    pub party_id: Option<Uuid>,
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
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct BrokenEggSale {
    pub id: Uuid,
    pub date: String,
    pub bakery_name: String,
    pub party_id: Option<Uuid>,
    pub trays_sold: i32,
    pub rate: f64,
    pub amount: f64,
    pub payment_received: f64,
    pub return_trays: i32,
    pub empty_trays_balance: i32,
    pub balance_amount: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateBrokenEggSale {
    pub date: String,
    pub bakery_name: String,
    pub party_id: Option<Uuid>,
    pub trays_sold: i32,
    pub rate: f64,
    pub amount: f64,
    pub payment_received: f64,
    pub return_trays: i32,
    pub empty_trays_balance: i32,
    pub balance_amount: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct MaterialPurchase {
    pub id: Uuid,
    pub date: String,
    pub material_name: String,
    pub party_name: String,
    pub party_id: Option<Uuid>,
    pub quantity_kg: f64,
    pub rate_per_kg: f64,
    pub total_amount: f64,
    pub advance_paid: f64,
    pub status: String,
    pub balance: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateMaterialPurchase {
    pub date: String,
    pub material_name: String,
    pub party_name: String,
    pub party_id: Option<Uuid>,
    pub quantity_kg: f64,
    pub rate_per_kg: f64,
    pub total_amount: f64,
    pub advance_paid: f64,
    pub status: String,
    pub balance: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct FeedBatch {
    pub id: Uuid,
    pub date: String,
    pub batch_id: String,
    pub feed_type: String,
    pub rate: f64,
    pub total_amount: f64,
    pub payment: f64,
    pub opening_balance: f64,
    pub closing_balance: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateFeedBatch {
    pub date: String,
    pub batch_id: String,
    pub feed_type: String,
    pub rate: f64,
    pub total_amount: f64,
    pub payment: f64,
    pub opening_balance: f64,
    pub closing_balance: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct LaborRecord {
    pub id: Uuid,
    pub date: String,
    pub employee_name: String,
    pub employee_id: Option<Uuid>,
    pub attendance: f64,
    pub advance_given: f64,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateLaborRecord {
    pub date: String,
    pub employee_name: String,
    pub employee_id: Option<Uuid>,
    pub attendance: f64,
    pub advance_given: f64,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct DailyProduction {
    pub id: Uuid,
    pub date: String,
    pub shed_name: String,
    pub egg_count_good: i32,
    pub egg_count_damaged: i32,
    pub mortality_count: i32,
    pub cull_count: i32,
    pub feed_consumed_kg: f64,
    pub notes: Option<String>,
    pub user_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateDailyProduction {
    pub date: String,
    pub shed_name: String,
    pub egg_count_good: i32,
    pub egg_count_damaged: i32,
    pub mortality_count: i32,
    pub cull_count: i32,
    pub feed_consumed_kg: f64,
    pub notes: Option<String>,
}
