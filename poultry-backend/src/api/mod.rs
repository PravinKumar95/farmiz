pub mod balance;
pub mod dashboard;
pub mod employees;
pub mod feed;
pub mod labor;
pub mod ledger;
pub mod models;
pub mod parties;
pub mod production;
pub mod purchases;
pub mod sales;

use axum::{
    routing::{get, put},
    Router,
};
use sqlx::PgPool;

pub fn routes() -> Router<PgPool> {
    Router::new()
        // Parties
        .route("/parties", get(parties::get_parties).post(parties::create_party))
        .route(
            "/parties/{id}",
            put(parties::update_party).delete(parties::delete_party),
        )
        .route("/parties/{id}/ledger", get(ledger::get_party_ledger))
        // Employees
        .route(
            "/employees",
            get(employees::get_employees).post(employees::create_employee),
        )
        .route(
            "/employees/{id}",
            put(employees::update_employee).delete(employees::delete_employee),
        )
        // Sales
        .route(
            "/sales/egg",
            get(sales::get_egg_sales).post(sales::create_egg_sale),
        )
        .route(
            "/sales/egg/{id}",
            put(sales::update_egg_sale).delete(sales::delete_egg_sale),
        )
        .route(
            "/sales/broken",
            get(sales::get_broken_sales).post(sales::create_broken_sale),
        )
        .route(
            "/sales/broken/{id}",
            put(sales::update_broken_sale).delete(sales::delete_broken_sale),
        )
        // Purchases
        .route(
            "/purchases",
            get(purchases::get_purchases).post(purchases::create_purchase),
        )
        .route(
            "/purchases/{id}",
            put(purchases::update_purchase).delete(purchases::delete_purchase),
        )
        // Labor
        .route(
            "/labor",
            get(labor::get_labor_records).post(labor::create_labor_record),
        )
        .route(
            "/labor/{id}",
            put(labor::update_labor_record).delete(labor::delete_labor_record),
        )
        // Feed
        .route(
            "/feed",
            get(feed::get_feed_batches).post(feed::create_feed_batch),
        )
        .route(
            "/feed/{id}",
            put(feed::update_feed_batch).delete(feed::delete_feed_batch),
        )
        // Daily Production & Flock Health
        .route(
            "/production",
            get(production::get_daily_production).post(production::create_daily_production),
        )
        .route(
            "/production/{id}",
            put(production::update_daily_production).delete(production::delete_daily_production),
        )
        // Dashboard
        .route("/dashboard/stats", get(dashboard::get_dashboard_stats))
}
