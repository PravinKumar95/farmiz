use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::balance::sync_party_balance;
use crate::api::models::{CreateMaterialPurchase, MaterialPurchase};
use crate::auth::AuthenticatedUser;

pub async fn get_purchases(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<MaterialPurchase>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, MaterialPurchase>(
        "SELECT * FROM material_purchases WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_purchase(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateMaterialPurchase>,
) -> Result<Json<MaterialPurchase>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, MaterialPurchase>(
        r#"INSERT INTO material_purchases (
            date, material_name, party_name, party_id, quantity_kg, rate_per_kg, total_amount, advance_paid, status, balance, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.material_name)
    .bind(&payload.party_name)
    .bind(payload.party_id)
    .bind(payload.quantity_kg)
    .bind(payload.rate_per_kg)
    .bind(payload.total_amount)
    .bind(payload.advance_paid)
    .bind(&payload.status)
    .bind(payload.balance)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.party_name, record.party_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn update_purchase(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<MaterialPurchase>,
) -> Result<Json<MaterialPurchase>, (StatusCode, String)> {
    let existing: Option<MaterialPurchase> = sqlx::query_as("SELECT * FROM material_purchases WHERE id=$1 AND user_id = $2")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    let record = sqlx::query_as::<_, MaterialPurchase>(
        r#"UPDATE material_purchases SET 
            date=$1, party_name=$2, party_id=$3, material_name=$4, quantity_kg=$5, 
            rate_per_kg=$6, total_amount=$7, advance_paid=$8, status=$9, balance=$10 
        WHERE id=$11 AND user_id = $12 RETURNING *"#,
    )
    .bind(p.date)
    .bind(&p.party_name)
    .bind(p.party_id)
    .bind(p.material_name)
    .bind(p.quantity_kg)
    .bind(p.rate_per_kg)
    .bind(p.total_amount)
    .bind(p.advance_paid)
    .bind(p.status)
    .bind(p.balance)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_party_balance(&pool, &record.party_name, record.party_id, &user.user_id).await;

    if let Some(old) = existing {
        if old.party_id != record.party_id || old.party_name != record.party_name {
            sync_party_balance(&pool, &old.party_name, old.party_id, &user.user_id).await;
        }
    }

    Ok(Json(record))
}

pub async fn delete_purchase(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing: Option<MaterialPurchase> = sqlx::query_as("SELECT * FROM material_purchases WHERE id=$1 AND user_id = $2")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    sqlx::query("DELETE FROM material_purchases WHERE id=$1 AND user_id = $2")
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
