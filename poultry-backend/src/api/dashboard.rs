use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use sqlx::PgPool;

use crate::api::models::DashboardStats;
use crate::auth::AuthenticatedUser;

pub async fn get_dashboard_stats(
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let today = Utc::now().format("%Y-%m-%d").to_string();

    let standard_sales: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(total_amount) FROM egg_sales WHERE date = $1 AND user_id = $2",
    )
    .bind(&today)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(Some(0.0));

    let broken_sales: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(amount) FROM broken_egg_sales WHERE date = $1 AND user_id = $2",
    )
    .bind(&today)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(Some(0.0));

    let today_sales = standard_sales.unwrap_or(0.0) + broken_sales.unwrap_or(0.0);

    let eggs_sold: Option<i64> = sqlx::query_scalar(
        "SELECT SUM(total_eggs) FROM egg_sales WHERE date = $1 AND user_id = $2",
    )
    .bind(&today)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(Some(0));

    let purchases: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(total_amount) FROM material_purchases WHERE date = $1 AND user_id = $2",
    )
    .bind(&today)
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(Some(0.0));

    let active_parties: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM parties WHERE user_id = $1",
    )
    .bind(&user.user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(Some(0));

    Ok(Json(DashboardStats {
        today_sales,
        eggs_sold_today: eggs_sold.unwrap_or(0) as i32,
        today_purchases: purchases.unwrap_or(0.0),
        active_parties: active_parties.unwrap_or(0),
    }))
}
