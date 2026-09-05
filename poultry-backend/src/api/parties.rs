use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::balance::sync_party_balance;
use crate::api::models::{CreateParty, Party};
use crate::auth::AuthenticatedUser;

pub async fn get_parties(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Party>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, Party>(
        "SELECT * FROM parties WHERE user_id = $1 ORDER BY created_at DESC",
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
    let existing: Option<Party> = sqlx::query_as("SELECT * FROM parties WHERE id = $1 AND user_id = $2")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    let _ = sqlx::query(
        "UPDATE parties SET name = $1, party_type = $2 WHERE id = $3 AND user_id = $4",
    )
    .bind(&payload.name)
    .bind(&payload.party_type)
    .bind(id)
    .bind(&user.user_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(ref old) = existing {
        if old.name != payload.name {
            let _ = sqlx::query("UPDATE egg_sales SET party_name = $1 WHERE (party_id = $2 OR party_name = $3) AND user_id = $4")
                .bind(&payload.name).bind(id).bind(&old.name).bind(&user.user_id).execute(&pool).await;
            let _ = sqlx::query("UPDATE broken_egg_sales SET bakery_name = $1 WHERE (party_id = $2 OR bakery_name = $3) AND user_id = $4")
                .bind(&payload.name).bind(id).bind(&old.name).bind(&user.user_id).execute(&pool).await;
            let _ = sqlx::query("UPDATE material_purchases SET party_name = $1 WHERE (party_id = $2 OR party_name = $3) AND user_id = $4")
                .bind(&payload.name).bind(id).bind(&old.name).bind(&user.user_id).execute(&pool).await;
        }
    }

    // Recalculate accurately from transactions
    sync_party_balance(&pool, &payload.name, Some(id), &user.user_id).await;

    let record = sqlx::query_as::<_, Party>("SELECT * FROM parties WHERE id = $1 AND user_id = $2")
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
    sqlx::query("DELETE FROM parties WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
