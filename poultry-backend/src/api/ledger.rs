use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::models::{BrokenEggSale, EggSale, LaborRecord, LedgerEntry, MaterialPurchase, Party};
use crate::auth::AuthenticatedUser;

pub async fn get_party_ledger(
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<LedgerEntry>>, (StatusCode, String)> {
    let party = sqlx::query_as::<_, Party>("SELECT * FROM parties WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(&user.user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut entries = Vec::new();

    let egg_sales = sqlx::query_as::<_, EggSale>("SELECT * FROM egg_sales WHERE (party_id = $1 OR (party_id IS NULL AND party_name = $2)) AND user_id = $3")
        .bind(id)
        .bind(&party.name)
        .bind(&user.user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for sale in egg_sales {
        entries.push(LedgerEntry {
            date: sale.date.clone(),
            description: format!("Egg Sale ({} boxes)", sale.quantity_boxes),
            charge: sale.total_amount,
            payment: sale.received_amount,
            created_at: sale.created_at,
        });
    }

    let broken_sales = sqlx::query_as::<_, BrokenEggSale>("SELECT * FROM broken_egg_sales WHERE (party_id = $1 OR (party_id IS NULL AND bakery_name = $2)) AND user_id = $3")
        .bind(id)
        .bind(&party.name)
        .bind(&user.user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for sale in broken_sales {
        entries.push(LedgerEntry {
            date: sale.date.clone(),
            description: format!("Broken Egg Sale ({} trays)", sale.trays_sold),
            charge: sale.amount,
            payment: sale.payment_received,
            created_at: sale.created_at,
        });
    }

    let purchases = sqlx::query_as::<_, MaterialPurchase>("SELECT * FROM material_purchases WHERE (party_id = $1 OR (party_id IS NULL AND party_name = $2)) AND user_id = $3")
        .bind(id)
        .bind(&party.name)
        .bind(&user.user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for p in purchases {
        entries.push(LedgerEntry {
            date: p.date.clone(),
            description: format!("Purchase ({})", p.material_name),
            charge: p.total_amount,
            payment: p.advance_paid,
            created_at: p.created_at,
        });
    }

    let labor = sqlx::query_as::<_, LaborRecord>("SELECT * FROM labor_records WHERE (employee_id = $1 OR (employee_id IS NULL AND employee_name = $2)) AND user_id = $3")
        .bind(id)
        .bind(&party.name)
        .bind(&user.user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    for l in labor {
        entries.push(LedgerEntry {
            date: l.date.clone(),
            description: format!("Labor (Attendance: {})", l.attendance),
            charge: 0.0,
            payment: l.advance_given,
            created_at: l.created_at,
        });
    }

    entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    Ok(Json(entries))
}
