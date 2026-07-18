use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// --- MODELS ---

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
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Employee {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub daily_wage: f64,
    pub current_balance: f64,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone)]
pub struct CreateEmployee {
    pub name: String,
    pub role: String,
    pub daily_wage: f64,
    pub current_balance: f64,
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
        .route("/parties/{id}/ledger", get(get_party_ledger))
        .route("/employees", get(get_employees).post(create_employee))
        .route("/employees/{id}", axum::routing::put(update_employee).delete(delete_employee))
        .route("/sales/egg", get(get_egg_sales).post(create_egg_sale))
        .route("/sales/egg/{id}", axum::routing::put(update_egg_sale).delete(delete_egg_sale))
        .route("/sales/broken", get(get_broken_sales).post(create_broken_sale))
        .route("/sales/broken/{id}", axum::routing::put(update_broken_sale).delete(delete_broken_sale))
        .route("/purchases", get(get_purchases).post(create_purchase))
        .route("/purchases/{id}", axum::routing::put(update_purchase).delete(delete_purchase))
        .route("/labor", get(get_labor_records).post(create_labor_record))
        .route("/labor/{id}", axum::routing::put(update_labor_record).delete(delete_labor_record))
        .route("/feed", get(get_feed_batches).post(create_feed_batch))
        .route("/feed/{id}", axum::routing::put(update_feed_batch).delete(delete_feed_batch))
        .route("/dashboard/stats", get(get_dashboard_stats))
}

// --- HANDLERS ---

async fn get_dashboard_stats(_user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    
    let standard_sales: Option<f64> = sqlx::query_scalar("SELECT SUM(total_amount) FROM egg_sales WHERE date = $1")
        .bind(&today).fetch_one(&pool).await.unwrap_or(Some(0.0));
        
    let broken_sales: Option<f64> = sqlx::query_scalar("SELECT SUM(amount) FROM broken_egg_sales WHERE date = $1")
        .bind(&today).fetch_one(&pool).await.unwrap_or(Some(0.0));
        
    let today_sales = standard_sales.unwrap_or(0.0) + broken_sales.unwrap_or(0.0);
    
    let eggs_sold: Option<i64> = sqlx::query_scalar("SELECT SUM(total_eggs) FROM egg_sales WHERE date = $1")
        .bind(&today).fetch_one(&pool).await.unwrap_or(Some(0));
        
    let purchases: Option<f64> = sqlx::query_scalar("SELECT SUM(total_amount) FROM material_purchases WHERE date = $1")
        .bind(&today).fetch_one(&pool).await.unwrap_or(Some(0.0));
        
    let active_parties: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM parties")
        .fetch_one(&pool).await.unwrap_or(Some(0));
        
    Ok(Json(DashboardStats {
        today_sales,
        eggs_sold_today: eggs_sold.unwrap_or(0) as i32,
        today_purchases: purchases.unwrap_or(0.0),
        active_parties: active_parties.unwrap_or(0),
    }))
}

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

async fn get_party_ledger(
    Path(id): Path<Uuid>,
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<LedgerEntry>>, (StatusCode, String)> {
    let party = sqlx::query_as::<_, Party>("SELECT * FROM parties WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut entries = Vec::new();

    let egg_sales = sqlx::query_as::<_, EggSale>("SELECT * FROM egg_sales WHERE party_name = $1")
        .bind(&party.name)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for sale in egg_sales {
        entries.push(LedgerEntry {
            date: sale.date.clone(),
            description: format!("Egg Sale ({} boxes)", sale.quantity_boxes),
            charge: sale.total_amount,
            payment: sale.received_amount,
            created_at: sale.created_at,
        });
    }

    let broken_sales = sqlx::query_as::<_, BrokenEggSale>("SELECT * FROM broken_egg_sales WHERE bakery_name = $1")
        .bind(&party.name)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for sale in broken_sales {
        entries.push(LedgerEntry {
            date: sale.date.clone(),
            description: format!("Broken Egg Sale ({} trays)", sale.trays_sold),
            charge: sale.amount,
            payment: sale.payment_received,
            created_at: sale.created_at,
        });
    }

    let purchases = sqlx::query_as::<_, MaterialPurchase>("SELECT * FROM material_purchases WHERE party_name = $1")
        .bind(&party.name)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for p in purchases {
        entries.push(LedgerEntry {
            date: p.date.clone(),
            description: format!("Purchase ({})", p.material_name),
            charge: p.total_amount,
            payment: p.advance_paid,
            created_at: p.created_at,
        });
    }

    let labor = sqlx::query_as::<_, LaborRecord>("SELECT * FROM labor_records WHERE employee_name = $1")
        .bind(&party.name)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for l in labor {
        entries.push(LedgerEntry {
            date: l.date.clone(),
            description: format!("Labor (Attendance: {})", l.attendance),
            charge: 0.0,
            payment: l.advance_given,
            created_at: l.created_at,
        });
    }

    entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(Json(entries))
}

async fn get_employees(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Employee>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, Employee>("SELECT * FROM employees ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

async fn create_employee(
    _user: crate::auth::AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Json<Employee>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Employee>(
        "INSERT INTO employees (name, role, daily_wage, current_balance) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.role)
    .bind(payload.daily_wage)
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

// EDIT / DELETE endpoints

async fn update_employee(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(payload): Json<Employee>) -> Result<Json<Employee>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Employee>("UPDATE employees SET name=$1, role=$2, daily_wage=$3, current_balance=$4 WHERE id=$5 RETURNING *")
        .bind(payload.name).bind(payload.role).bind(payload.daily_wage).bind(payload.current_balance).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_employee(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM employees WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_egg_sale(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(p): Json<EggSale>) -> Result<Json<EggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, EggSale>("UPDATE egg_sales SET date=$1, party_name=$2, quantity_boxes=$3, total_eggs=$4, size=$5, gross_rate=$6, less_discount=$7, net_rate=$8, total_amount=$9, received_amount=$10, payment_mode=$11, balance=$12 WHERE id=$13 RETURNING *")
        .bind(p.date).bind(p.party_name).bind(p.quantity_boxes).bind(p.total_eggs).bind(p.size).bind(p.gross_rate).bind(p.less_discount).bind(p.net_rate).bind(p.total_amount).bind(p.received_amount).bind(p.payment_mode).bind(p.balance).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_egg_sale(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM egg_sales WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_broken_sale(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(p): Json<BrokenEggSale>) -> Result<Json<BrokenEggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, BrokenEggSale>("UPDATE broken_egg_sales SET date=$1, bakery_name=$2, trays_sold=$3, rate=$4, amount=$5, payment_received=$6, return_trays=$7, empty_trays_balance=$8, balance_amount=$9 WHERE id=$10 RETURNING *")
        .bind(p.date).bind(p.bakery_name).bind(p.trays_sold).bind(p.rate).bind(p.amount).bind(p.payment_received).bind(p.return_trays).bind(p.empty_trays_balance).bind(p.balance_amount).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_broken_sale(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM broken_egg_sales WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_purchase(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(p): Json<MaterialPurchase>) -> Result<Json<MaterialPurchase>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, MaterialPurchase>("UPDATE material_purchases SET date=$1, party_name=$2, material_name=$3, quantity_kg=$4, rate_per_kg=$5, total_amount=$6, advance_paid=$7, status=$8, balance=$9 WHERE id=$10 RETURNING *")
        .bind(p.date).bind(p.party_name).bind(p.material_name).bind(p.quantity_kg).bind(p.rate_per_kg).bind(p.total_amount).bind(p.advance_paid).bind(p.status).bind(p.balance).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_purchase(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM material_purchases WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_labor_record(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(p): Json<LaborRecord>) -> Result<Json<LaborRecord>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, LaborRecord>("UPDATE labor_records SET date=$1, employee_name=$2, attendance=$3, advance_given=$4 WHERE id=$5 RETURNING *")
        .bind(p.date).bind(p.employee_name).bind(p.attendance).bind(p.advance_given).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_labor_record(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM labor_records WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_feed_batch(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>, Json(p): Json<FeedBatch>) -> Result<Json<FeedBatch>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, FeedBatch>("UPDATE feed_batches SET date=$1, batch_id=$2, feed_type=$3, rate=$4, total_amount=$5, payment=$6, opening_balance=$7, closing_balance=$8 WHERE id=$9 RETURNING *")
        .bind(p.date).bind(p.batch_id).bind(p.feed_type).bind(p.rate).bind(p.total_amount).bind(p.payment).bind(p.opening_balance).bind(p.closing_balance).bind(id)
        .fetch_one(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}
async fn delete_feed_batch(Path(id): Path<Uuid>, _user: crate::auth::AuthenticatedUser, State(pool): State<PgPool>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM feed_batches WHERE id=$1").bind(id).execute(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
