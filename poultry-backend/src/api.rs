use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// --- MODELS ---

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Party {
    pub id: Uuid,
    pub name: String,
    pub party_type: String,
    pub current_balance: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateParty {
    pub name: String,
    pub party_type: String,
    pub current_balance: f64,
}


#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct EggSale {
    pub id: Uuid,
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
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateEggSale {
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
}


#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct BrokenEggSale {
    pub id: Uuid,
    pub date: String,
    pub bakery_name: String,
    pub trays_sold: i32,
    pub rate: f64,
    pub amount: f64,
    pub payment_received: f64,
    pub return_trays: i32,
    pub empty_trays_balance: i32,
    pub balance_amount: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateBrokenEggSale {
    pub date: String,
    pub bakery_name: String,
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
    pub quantity_kg: f64,
    pub rate_per_kg: f64,
    pub total_amount: f64,
    pub advance_paid: f64,
    pub status: String,
    pub balance: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateMaterialPurchase {
    pub date: String,
    pub material_name: String,
    pub party_name: String,
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
    pub attendance: f64,
    pub advance_given: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateLaborRecord {
    pub date: String,
    pub employee_name: String,
    pub attendance: f64,
    pub advance_given: f64,
}


// --- ROUTER ---

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/parties", get(get_parties).post(create_party))
        .route("/sales/egg", get(get_egg_sales).post(create_egg_sale))
        .route("/sales/broken", get(get_broken_sales).post(create_broken_sale))
        .route("/purchases", get(get_purchases).post(create_purchase))
        .route("/feed", get(get_feed_batches).post(create_feed_batch))
        .route("/labor", get(get_labor_records).post(create_labor_record))
}

// --- HANDLERS ---

async fn get_parties(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<Party>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, Party>("SELECT * FROM parties ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_party(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateParty>,
) -> Result<Json<Party>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Party>(
        "INSERT INTO parties (name, party_type, current_balance) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.party_type)
    .bind(payload.current_balance)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

async fn get_egg_sales(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<EggSale>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, EggSale>("SELECT * FROM egg_sales ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_egg_sale(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateEggSale>,
) -> Result<Json<EggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, EggSale>(
        r#"INSERT INTO egg_sales (
            date, party_name, quantity_boxes, total_eggs, size, gross_rate, 
            less_discount, net_rate, total_amount, received_amount, payment_mode, balance
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *"#
    )
    .bind(&payload.date).bind(&payload.party_name).bind(payload.quantity_boxes).bind(payload.total_eggs)
    .bind(&payload.size).bind(payload.gross_rate).bind(payload.less_discount).bind(payload.net_rate)
    .bind(payload.total_amount).bind(payload.received_amount).bind(&payload.payment_mode).bind(payload.balance)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

async fn get_broken_sales(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<BrokenEggSale>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, BrokenEggSale>("SELECT * FROM broken_egg_sales ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_broken_sale(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBrokenEggSale>,
) -> Result<Json<BrokenEggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, BrokenEggSale>(
        r#"INSERT INTO broken_egg_sales (
            date, bakery_name, trays_sold, rate, amount, payment_received, return_trays, empty_trays_balance, balance_amount
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#
    )
    .bind(&payload.date).bind(&payload.bakery_name).bind(payload.trays_sold).bind(payload.rate)
    .bind(payload.amount).bind(payload.payment_received).bind(payload.return_trays)
    .bind(payload.empty_trays_balance).bind(payload.balance_amount)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

async fn get_purchases(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<MaterialPurchase>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, MaterialPurchase>("SELECT * FROM material_purchases ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_purchase(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateMaterialPurchase>,
) -> Result<Json<MaterialPurchase>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, MaterialPurchase>(
        r#"INSERT INTO material_purchases (
            date, material_name, party_name, quantity_kg, rate_per_kg, total_amount, advance_paid, status, balance
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#
    )
    .bind(&payload.date).bind(&payload.material_name).bind(&payload.party_name)
    .bind(payload.quantity_kg).bind(payload.rate_per_kg).bind(payload.total_amount)
    .bind(payload.advance_paid).bind(&payload.status).bind(payload.balance)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

async fn get_feed_batches(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<FeedBatch>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, FeedBatch>("SELECT * FROM feed_batches ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_feed_batch(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateFeedBatch>,
) -> Result<Json<FeedBatch>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, FeedBatch>(
        r#"INSERT INTO feed_batches (
            date, batch_id, feed_type, rate, total_amount, payment, opening_balance, closing_balance
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#
    )
    .bind(&payload.date).bind(&payload.batch_id).bind(&payload.feed_type).bind(payload.rate)
    .bind(payload.total_amount).bind(payload.payment).bind(payload.opening_balance).bind(payload.closing_balance)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

async fn get_labor_records(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<Vec<LaborRecord>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, LaborRecord>("SELECT * FROM labor_records ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_labor_record(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateLaborRecord>,
) -> Result<Json<LaborRecord>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, LaborRecord>(
        r#"INSERT INTO labor_records (
            date, employee_name, attendance, advance_given
        ) VALUES ($1, $2, $3, $4) RETURNING *"#
    )
    .bind(&payload.date).bind(&payload.employee_name).bind(payload.attendance).bind(payload.advance_given)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
