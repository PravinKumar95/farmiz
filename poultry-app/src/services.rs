use dioxus::prelude::*;
use crate::models::{
    get_dummy_broken_egg_sales, get_dummy_egg_sales, get_dummy_feed_batches,
    get_dummy_labor_records, get_dummy_material_purchases, get_dummy_parties, BrokenEggSale,
    EggSale, FeedBatch, LaborRecord, MaterialPurchase, Party,
};

/// Service Layer to loosely couple UI from the actual Data Models and data source.
/// These hooks will later encapsulate API calls and state management (e.g., using `use_resource`).

pub fn use_egg_sales() -> Signal<Vec<EggSale>> {
    use_signal(|| get_dummy_egg_sales())
}

pub fn use_broken_egg_sales() -> Signal<Vec<BrokenEggSale>> {
    use_signal(|| get_dummy_broken_egg_sales())
}

pub fn use_material_purchases() -> Signal<Vec<MaterialPurchase>> {
    use_signal(|| get_dummy_material_purchases())
}

pub fn use_feed_batches() -> Signal<Vec<FeedBatch>> {
    use_signal(|| get_dummy_feed_batches())
}

pub fn use_labor_records() -> Signal<Vec<LaborRecord>> {
    use_signal(|| get_dummy_labor_records())
}

pub fn use_parties() -> Signal<Vec<Party>> {
    use_signal(|| get_dummy_parties())
}
