use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::models::{CreateEmployee, Employee};
use crate::auth::AuthenticatedUser;

pub async fn get_employees(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Employee>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, Employee>(
        "SELECT * FROM employees WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(&user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(records))
}

pub async fn create_employee(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Json<Employee>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Employee>(
        "INSERT INTO employees (name, role, daily_wage, current_balance, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(&payload.name)
    .bind(&payload.role)
    .bind(payload.daily_wage)
    .bind(payload.current_balance)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn update_employee(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
    Json(payload): Json<Employee>,
) -> Result<Json<Employee>, (StatusCode, String)> {
    let record = sqlx::query_as::<_, Employee>(
        "UPDATE employees SET name=$1, role=$2, daily_wage=$3, current_balance=$4 WHERE id=$5 AND user_id = $6 RETURNING *",
    )
    .bind(payload.name)
    .bind(payload.role)
    .bind(payload.daily_wage)
    .bind(payload.current_balance)
    .bind(id)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(record))
}

pub async fn delete_employee(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM employees WHERE id=$1 AND user_id = $2")
        .bind(id)
        .bind(&user.user_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
