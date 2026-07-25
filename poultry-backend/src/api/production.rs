use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::models::{CreateDailyProduction, DailyProduction};
use crate::auth::AuthenticatedUser;

pub async fn get_daily_production(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<DailyProduction>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, DailyProduction>(
        "SELECT * FROM daily_production WHERE user_id = $1 OR user_id IS NULL ORDER BY date DESC, created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_daily_production(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateDailyProduction>,
) -> Result<Json<DailyProduction>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, DailyProduction>(
        r#"INSERT INTO daily_production (
            date, shed_name, egg_count_good, egg_count_damaged, mortality_count, cull_count, feed_consumed_kg, notes, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.shed_name)
    .bind(payload.egg_count_good)
    .bind(payload.egg_count_damaged)
    .bind(payload.mortality_count)
    .bind(payload.cull_count)
    .bind(payload.feed_consumed_kg)
    .bind(&payload.notes)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(record))
}

pub async fn update_daily_production(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<DailyProduction>,
) -> Result<Json<DailyProduction>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, DailyProduction>(
        r#"UPDATE daily_production SET 
            date=$1, shed_name=$2, egg_count_good=$3, egg_count_damaged=$4, mortality_count=$5, 
            cull_count=$6, feed_consumed_kg=$7, notes=$8 
        WHERE id=$9 AND (user_id = $10 OR user_id IS NULL) RETURNING *"#,
    )
    .bind(p.date)
    .bind(p.shed_name)
    .bind(p.egg_count_good)
    .bind(p.egg_count_damaged)
    .bind(p.mortality_count)
    .bind(p.cull_count)
    .bind(p.feed_consumed_kg)
    .bind(p.notes)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(record))
}

pub async fn delete_daily_production(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM daily_production WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
