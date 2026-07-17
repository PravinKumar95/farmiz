use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EggSale {
    pub id: String,
    pub date: String,
    pub party_name: String,
    pub quantity_boxes: u32,
    pub total_eggs: u32,
    pub size: String,
    pub gross_rate: f64,
    pub less_discount: f64,
    pub net_rate: f64,
    pub total_amount: f64,
    pub received_amount: f64,
    pub payment_mode: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrokenEggSale {
    pub id: String,
    pub date: String,
    pub bakery_name: String,
    pub trays_sold: u32,
    pub rate: f64,
    pub amount: f64,
    pub payment_received: f64,
    pub return_trays: u32,
    pub empty_trays_balance: u32,
    pub balance_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialPurchase {
    pub id: String,
    pub date: String,
    pub material_name: String,
    pub party_name: String,
    pub quantity_kg: f64,
    pub rate_per_kg: f64,
    pub total_amount: f64,
    pub advance_paid: f64,
    pub status: String, // "PAID" or "PENDING"
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedBatch {
    pub id: String,
    pub date: String,
    pub batch_id: String,
    pub feed_type: String,
    pub rate: f64,
    pub total_amount: f64,
    pub payment: f64,
    pub opening_balance: f64,
    pub closing_balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaborRecord {
    pub id: String,
    pub date: String,
    pub employee_name: String,
    pub attendance: f64, // 1.0, 0.5, 0.0
    pub advance_given: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub party_type: String, // "CUSTOMER", "SUPPLIER", "EMPLOYEE", "BAKERY"
    pub current_balance: f64,
}

// Dummy data for initial UI rendering
pub fn get_dummy_egg_sales() -> Vec<EggSale> {
    vec![
        EggSale {
            id: "1".into(),
            date: "21.1.26".into(),
            party_name: "AKG".into(),
            quantity_boxes: 1293,
            total_eggs: 38790,
            size: "LARGE".into(),
            gross_rate: 5.0,
            less_discount: 0.47,
            net_rate: 4.53,
            total_amount: 175718.7,
            received_amount: 400000.0,
            payment_mode: "CASH".into(),
            balance: -224281.3,
        }
    ]
}

pub fn get_dummy_broken_egg_sales() -> Vec<BrokenEggSale> {
    vec![
        BrokenEggSale {
            id: "1".into(),
            date: "28.4.25".into(),
            bakery_name: "SURIYA BAKERY".into(),
            trays_sold: 10,
            rate: 70.0,
            amount: 700.0,
            payment_received: 230.0,
            return_trays: 10,
            empty_trays_balance: 0,
            balance_amount: 470.0,
        }
    ]
}

pub fn get_dummy_material_purchases() -> Vec<MaterialPurchase> {
    vec![
        MaterialPurchase {
            id: "1".into(),
            date: "1.1.26".into(),
            material_name: "Maize".into(),
            party_name: "Silambarasan".into(),
            quantity_kg: 21420.0,
            rate_per_kg: 20.6,
            total_amount: 441252.0,
            advance_paid: 51720.0,
            status: "PAID".into(),
            balance: 389532.0,
        }
    ]
}

pub fn get_dummy_feed_batches() -> Vec<FeedBatch> {
    vec![
        FeedBatch {
            id: "1".into(),
            date: "28.1.26".into(),
            batch_id: "B-001".into(),
            feed_type: "LAYER MASH".into(),
            rate: 25.0,
            total_amount: 5000.0,
            payment: 200.0,
            opening_balance: -33520.0,
            closing_balance: -33720.0,
        }
    ]
}

pub fn get_dummy_labor_records() -> Vec<LaborRecord> {
    vec![
        LaborRecord {
            id: "1".into(),
            date: "16.6.26".into(),
            employee_name: "RAJENDER".into(),
            attendance: 1.0,
            advance_given: 4813.0,
        }
    ]
}

pub fn get_dummy_parties() -> Vec<Party> {
    vec![
        Party {
            id: "1".into(),
            name: "AKG".into(),
            party_type: "CUSTOMER".into(),
            current_balance: -224281.3,
        },
        Party {
            id: "2".into(),
            name: "Silambarasan".into(),
            party_type: "SUPPLIER".into(),
            current_balance: 389532.0,
        },
    ]
}
