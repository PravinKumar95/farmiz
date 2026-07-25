use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::models::{CreateParty, Party};
use crate::auth::AuthenticatedUser;

pub async fn get_parties(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Party>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, Party>(
        "SELECT * FROM parties WHERE user_id = $1 OR user_id IS NULL ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_party(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateParty>,
) -> Result<Json<Party>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Party>(
        "INSERT INTO parties (name, party_type, current_balance, user_id) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.party_type)
    .bind(payload.current_balance)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn update_party(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<Party>,
) -> Result<Json<Party>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Party>(
        "UPDATE parties SET name = $1, party_type = $2, current_balance = $3 WHERE id = $4 AND (user_id = $5 OR user_id IS NULL) RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.party_type)
    .bind(payload.current_balance)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn delete_party(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM parties WHERE id = $1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
