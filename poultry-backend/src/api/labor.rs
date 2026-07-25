use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::balance::sync_employee_balance;
use crate::api::models::{CreateLaborRecord, LaborRecord};
use crate::auth::AuthenticatedUser;

pub async fn get_labor_records(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<LaborRecord>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, LaborRecord>(
        "SELECT * FROM labor_records WHERE user_id = $1 OR user_id IS NULL ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_labor_record(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateLaborRecord>,
) -> Result<Json<LaborRecord>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, LaborRecord>(
        r#"INSERT INTO labor_records (
            date, employee_name, employee_id, attendance, advance_given, user_id
        ) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
    )
    .bind(&payload.date)
    .bind(&payload.employee_name)
    .bind(payload.employee_id)
    .bind(payload.attendance)
    .bind(payload.advance_given)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_employee_balance(&pool, &record.employee_name, record.employee_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn update_labor_record(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(p): Json<LaborRecord>,
) -> Result<Json<LaborRecord>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, LaborRecord>(
        r#"UPDATE labor_records SET 
            date=$1, employee_name=$2, employee_id=$3, attendance=$4, advance_given=$5 
        WHERE id=$6 AND (user_id = $7 OR user_id IS NULL) RETURNING *"#,
    )
    .bind(p.date)
    .bind(&p.employee_name)
    .bind(p.employee_id)
    .bind(p.attendance)
    .bind(p.advance_given)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sync_employee_balance(&pool, &record.employee_name, record.employee_id, &user.user_id).await;

    Ok(Json(record))
}

pub async fn delete_labor_record(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing: Option<LaborRecord> = sqlx::query_as("SELECT * FROM labor_records WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id).bind(&user.user_id).fetch_optional(&pool).await.unwrap_or(None);

    sqlx::query("DELETE FROM labor_records WHERE id=$1 AND (user_id = $2 OR user_id IS NULL)")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(record) = existing {
        sync_employee_balance(&pool, &record.employee_name, record.employee_id, &user.user_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
