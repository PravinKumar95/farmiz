use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::balance::sync_party_balance;
use crate::api::models::{BrokenEggSale, CreateBrokenEggSale, CreateEggSale, EggSale};
use crate::auth::AuthenticatedUser;

// --- EGG SALES ---

pub async fn get_egg_sales(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<EggSale>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, EggSale>(
        "SELECT * FROM egg_sales WHERE user_id = $1 OR user_id IS NULL ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_egg_sale(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateEggSale>,
) -> Result<Json<EggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, EggSale>(
        r#"INSERT INTO egg_sales (
            date, party_name, party_id, quantity_boxes, total_eggs, size, gross_rate, 
            less_discount, net_rate, total_amount, received_amount, payment_mode, balance, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.party_name)
    .bind(payload.party_id)
    .bind(payload.quantity_boxes)
    .bind(payload.total_eggs)
    .bind(&payload.size)
    .bind(payload.gross_rate)
    .bind(payload.less_discount)
    .bind(payload.net_rate)
    .bind(payload.total_amount)
    .bind(payload.received_amount)
    .bind(&payload.payment_mode)
    .bind(payload.balance)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.party_name, record.party_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn update_egg_sale(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<EggSale>,
) -> Result<Json<EggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, EggSale>(
        r#"UPDATE egg_sales SET 
            date=$1, party_name=$2, party_id=$3, quantity_boxes=$4, total_eggs=$5, size=$6, 
            gross_rate=$7, less_discount=$8, net_rate=$9, total_amount=$10, received_amount=$11, 
            payment_mode=$12, balance=$13 
        WHERE id=$14 AND (user_id = $15 OR user_id IS NULL) RETURNING *"#,
    )
    .bind(p.date)
    .bind(&p.party_name)
    .bind(p.party_id)
    .bind(p.quantity_boxes)
    .bind(p.total_eggs)
    .bind(p.size)
    .bind(p.gross_rate)
    .bind(p.less_discount)
    .bind(p.net_rate)
    .bind(p.total_amount)
    .bind(p.received_amount)
    .bind(p.payment_mode)
    .bind(p.balance)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.party_name, record.party_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn delete_egg_sale(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing: Option<EggSale> = sqlx::query_as("SELECT * FROM egg_sales WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    sqlx::query("DELETE FROM egg_sales WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(record) = existing {
        sync_party_balance(&pool, &record.party_name, record.party_id, &user.user_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- BROKEN EGG SALES ---

pub async fn get_broken_sales(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<BrokenEggSale>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, BrokenEggSale>(
        "SELECT * FROM broken_egg_sales WHERE user_id = $1 OR user_id IS NULL ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_broken_sale(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBrokenEggSale>,
) -> Result<Json<BrokenEggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, BrokenEggSale>(
        r#"INSERT INTO broken_egg_sales (
            date, bakery_name, party_id, trays_sold, rate, amount, payment_received, return_trays, empty_trays_balance, balance_amount, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.bakery_name)
    .bind(payload.party_id)
    .bind(payload.trays_sold)
    .bind(payload.rate)
    .bind(payload.amount)
    .bind(payload.payment_received)
    .bind(payload.return_trays)
    .bind(payload.empty_trays_balance)
    .bind(payload.balance_amount)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.bakery_name, record.party_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn update_broken_sale(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<BrokenEggSale>,
) -> Result<Json<BrokenEggSale>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, BrokenEggSale>(
        r#"UPDATE broken_egg_sales SET 
            date=$1, bakery_name=$2, party_id=$3, trays_sold=$4, rate=$5, amount=$6, 
            payment_received=$7, return_trays=$8, empty_trays_balance=$9, balance_amount=$10 
        WHERE id=$11 AND (user_id = $12 OR user_id IS NULL) RETURNING *"#,
    )
    .bind(p.date)
    .bind(&p.bakery_name)
    .bind(p.party_id)
    .bind(p.trays_sold)
    .bind(p.rate)
    .bind(p.amount)
    .bind(p.payment_received)
    .bind(p.return_trays)
    .bind(p.empty_trays_balance)
    .bind(p.balance_amount)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.bakery_name, record.party_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn delete_broken_sale(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing: Option<BrokenEggSale> = sqlx::query_as("SELECT * FROM broken_egg_sales WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    sqlx::query("DELETE FROM broken_egg_sales WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(record) = existing {
        sync_party_balance(&pool, &record.bakery_name, record.party_id, &user.user_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
