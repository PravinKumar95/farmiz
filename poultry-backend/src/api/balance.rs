use sqlx::PgPool;
use uuid::Uuid;

/// Recalculates and updates the current_balance of a Party by summing up all associated transactions.
pub async fn sync_party_balance(pool: &PgPool, party_name: &str, party_id: Option<Uuid>, user_id: &str) {
    // 1. Fetch party record to determine party_type
    let party_opt: Option<(Uuid, String, String)> = if let Some(pid) = party_id {
        sqlx::query_as("SELECT id, name, party_type FROM parties WHERE id = $1 AND user_id = $2")
            .bind(pid).bind(user_id).fetch_optional(pool).await.unwrap_or(None)
    } else {
        sqlx::query_as("SELECT id, name, party_type FROM parties WHERE name = $1 AND user_id = $2")
            .bind(party_name).bind(user_id).fetch_optional(pool).await.unwrap_or(None)
    };

    let (target_id, target_name, party_type) = match party_opt {
        Some((id, name, ptype)) => (Some(id), name, ptype),
        None => (party_id, party_name.to_string(), "CUSTOMER".to_string()),
    };

    let total_balance = if party_type == "SUPPLIER" {
        // Material Purchases: advance_paid - total_amount
        // (Negative means farm owes money to supplier; positive means advance surplus)
        let purchase_balance_res: Option<f64> = if let Some(pid) = target_id {
            sqlx::query_scalar("SELECT SUM(advance_paid - total_amount) FROM material_purchases WHERE (party_id = $1 OR (party_id IS NULL AND party_name = $2)) AND user_id = $3")
                .bind(pid).bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        } else {
            sqlx::query_scalar("SELECT SUM(advance_paid - total_amount) FROM material_purchases WHERE party_name = $1 AND user_id = $2")
                .bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        };
        purchase_balance_res.unwrap_or(0.0)
    } else {
        // Customers / Bakeries: total_amount - received_amount
        // (Positive means customer owes money to farm; negative means customer overpayment)
        let egg_charges_res: Option<f64> = if let Some(pid) = target_id {
            sqlx::query_scalar("SELECT SUM(total_amount - received_amount) FROM egg_sales WHERE (party_id = $1 OR (party_id IS NULL AND party_name = $2)) AND user_id = $3")
                .bind(pid).bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        } else {
            sqlx::query_scalar("SELECT SUM(total_amount - received_amount) FROM egg_sales WHERE party_name = $1 AND user_id = $2")
                .bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        };

        let broken_charges_res: Option<f64> = if let Some(pid) = target_id {
            sqlx::query_scalar("SELECT SUM(amount - payment_received) FROM broken_egg_sales WHERE (party_id = $1 OR (party_id IS NULL AND bakery_name = $2)) AND user_id = $3")
                .bind(pid).bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        } else {
            sqlx::query_scalar("SELECT SUM(amount - payment_received) FROM broken_egg_sales WHERE bakery_name = $1 AND user_id = $2")
                .bind(&target_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
        };

        egg_charges_res.unwrap_or(0.0) + broken_charges_res.unwrap_or(0.0)
    };

    // Update parties table
    if let Some(pid) = target_id {
        let _ = sqlx::query("UPDATE parties SET current_balance = $1 WHERE id = $2 AND user_id = $3")
            .bind(total_balance).bind(pid).bind(user_id).execute(pool).await;
    } else {
        let _ = sqlx::query("UPDATE parties SET current_balance = $1 WHERE name = $2 AND user_id = $3")
            .bind(total_balance).bind(&target_name).bind(user_id).execute(pool).await;
    }
}

/// Recalculates and updates the current_balance of an Employee by summing up labor advances.
pub async fn sync_employee_balance(pool: &PgPool, employee_name: &str, employee_id: Option<Uuid>, user_id: &str) {
    let advances_res: Option<f64> = if let Some(eid) = employee_id {
        sqlx::query_scalar("SELECT SUM(advance_given) FROM labor_records WHERE (employee_id = $1 OR (employee_id IS NULL AND employee_name = $2)) AND user_id = $3")
            .bind(eid).bind(employee_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
    } else {
        sqlx::query_scalar("SELECT SUM(advance_given) FROM labor_records WHERE employee_name = $1 AND user_id = $2")
            .bind(employee_name).bind(user_id).fetch_one(pool).await.unwrap_or(None)
    };

    let total_advances = advances_res.unwrap_or(0.0);

    if let Some(eid) = employee_id {
        let _ = sqlx::query("UPDATE employees SET current_balance = $1 WHERE id = $2 AND user_id = $3")
            .bind(total_advances).bind(eid).bind(user_id).execute(pool).await;
    } else {
        let _ = sqlx::query("UPDATE employees SET current_balance = $1 WHERE name = $2 AND user_id = $3")
            .bind(total_advances).bind(employee_name).bind(user_id).execute(pool).await;
    }
}
