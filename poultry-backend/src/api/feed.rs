use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::models::{CreateFeedBatch, FeedBatch};
use crate::auth::AuthenticatedUser;

pub async fn get_feed_batches(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<FeedBatch>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, FeedBatch>(
        "SELECT * FROM feed_batches WHERE user_id = $1 OR user_id IS NULL ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_feed_batch(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateFeedBatch>,
) -> Result<Json<FeedBatch>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, FeedBatch>(
        r#"INSERT INTO feed_batches (
            date, batch_id, feed_type, rate, total_amount, payment, opening_balance, closing_balance, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.batch_id)
    .bind(&payload.feed_type)
    .bind(payload.rate)
    .bind(payload.total_amount)
    .bind(payload.payment)
    .bind(payload.opening_balance)
    .bind(payload.closing_balance)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn update_feed_batch(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<FeedBatch>,
) -> Result<Json<FeedBatch>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, FeedBatch>(
        r#"UPDATE feed_batches SET 
            date=$1, batch_id=$2, feed_type=$3, rate=$4, total_amount=$5, payment=$6, 
            opening_balance=$7, closing_balance=$8 
        WHERE id=$9 AND (user_id = $10 OR user_id IS NULL) RETURNING *"#,
    )
    .bind(p.date)
    .bind(p.batch_id)
    .bind(p.feed_type)
    .bind(p.rate)
    .bind(p.total_amount)
    .bind(p.payment)
    .bind(p.opening_balance)
    .bind(p.closing_balance)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn delete_feed_batch(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM feed_batches WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
