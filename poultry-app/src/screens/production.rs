use dioxus::prelude::*;
use chrono::Utc;
use crate::components::toast::{use_toast, ToastOptions};
use crate::i18n::tr;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::Card;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::date_range_filter::{DateRange, DateRangeFilter};
use crate::components::layout_toggle::{LayoutToggle, ViewLayout};
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::DailyProduction;
use crate::services::*;

#[component]
pub fn Production() -> Element {
    let mut logs = use_daily_production();
    let api = use_auth();
    let toast_api = use_toast();

    // UI state
    let mut view_layout = use_signal(ViewLayout::default);
    let mut is_sheet_open = use_signal(|| false);
    let mut is_manage_batches_open = use_signal(|| false);

    // Available batches list
    // Available batches list & persistent flock sizes
    let mut available_batches = use_signal(|| vec![
        "Batch A".to_string(),
        "Batch B".to_string(),
        "Batch C".to_string(),
        "Batch D".to_string(),
        "Batch E".to_string(),
        "Batch F".to_string(),
    ]);
    let mut batch_flock_sizes = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, std::collections::HashMap<String, u32>>(
        "batch_flock_sizes".to_string(),
        || std::collections::HashMap::new(),
    );
    let mut new_batch_name_input = use_signal(String::new);

    // Filters
    let mut date_range = use_signal(DateRange::default);
    let mut selected_batch = use_signal(|| Option::<String>::None); // None = All Batches
    let mut search_query = use_signal(String::new);

    // Dialog & Form states
    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let today_str = Utc::now().format("%Y-%m-%d").to_string();

    let mut form_date = use_signal(move || today_str.clone());
    let mut form_shed = use_signal(|| "Batch A".to_string());
    let mut is_first_entry = use_signal(|| false);
    let mut form_opening_flock = use_signal(|| 0.0f64);
    let mut form_flock_size = use_signal(String::new);
    let mut form_prev_stock = use_signal(|| 0.0f64);
    let mut form_trays = use_signal(|| "0".to_string());
    let mut form_good = use_signal(|| "0".to_string());
    let mut form_damaged = use_signal(|| "0".to_string());
    let mut form_dirty1 = use_signal(|| "0".to_string());
    let mut form_dirty2 = use_signal(|| "0".to_string());
    let mut form_yield = use_signal(|| "0.0".to_string());
    let mut form_stock = use_signal(|| "0".to_string());
    let mut form_mortality = use_signal(|| "0".to_string());
    let mut form_cull = use_signal(|| "0".to_string());
    let mut form_notes = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Helper: Find latest reported closing stock for a given batch/shed
    let get_latest_stock_for_shed = move |shed: &str| -> f64 {
        let list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
        list.iter()
            .filter(|r| r.shed_name.eq_ignore_ascii_case(shed))
            .map(|r| r.stock_in_trays)
            .next()
            .unwrap_or(0.0)
    };

    // Helper: Lookup initial flock size for a batch from stored config or past records
    let get_initial_flock_for_shed = move |shed: &str| -> Option<u32> {
        let sizes = batch_flock_sizes();
        if let Some(&size) = sizes.get(shed) {
            if size > 0 {
                return Some(size);
            }
        }
        let list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
        let mut shed_records: Vec<_> = list.into_iter()
            .filter(|r| r.shed_name.eq_ignore_ascii_case(shed))
            .collect();
        shed_records.sort_by(|a, b| a.date.cmp(&b.date));
        if let Some(first_rec) = shed_records.first() {
            if first_rec.flock_size > 0 {
                let starting = first_rec.flock_size as u32 + (first_rec.mortality_count + first_rec.cull_count) as u32;
                return Some(starting);
            }
            if first_rec.production_percentage > 0.0 && first_rec.egg_count_good > 0 {
                let deduced = ((first_rec.egg_count_good as f64) / (first_rec.production_percentage / 100.0)).round() as u32;
                if deduced > 0 {
                    return Some(deduced);
                }
            }
        }
        None
    };

    // Helper: Determine if this is the first entry for this batch
    let is_first_entry_for_shed = move |shed: &str, current_id: Option<&str>| -> bool {
        let list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
        let other_records_count = list.iter()
            .filter(|r| r.shed_name.eq_ignore_ascii_case(shed))
            .filter(|r| {
                if let Some(cid) = current_id {
                    r.id != cid
                } else {
                    true
                }
            })
            .count();
        other_records_count == 0 || get_initial_flock_for_shed(shed).is_none()
    };

    // Helper: Calculate opening flock (initial batch size minus past mortality & culls)
    let get_opening_flock_for_shed = move |shed: &str, current_edit_id: Option<&str>| -> f64 {
        let initial = get_initial_flock_for_shed(shed).unwrap_or(0) as f64;
        let list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
        let past_attrition: f64 = list.iter()
            .filter(|r| r.shed_name.eq_ignore_ascii_case(shed))
            .filter(|r| {
                if let Some(cid) = current_edit_id {
                    r.id != cid
                } else {
                    true
                }
            })
            .map(|r| (r.mortality_count + r.cull_count) as f64)
            .sum();
        (initial - past_attrition).max(0.0)
    };

    // Helper: Recalculate live flock size and laying yield % from trays, mortality, and culls
    let mut recalc_flock_and_yield = move |trays_val: &str, mort_val: &str, cull_val: &str| {
        let trays = trays_val.parse::<f64>().unwrap_or(0.0);
        let eggs = (trays * 30.0).round();
        let mort = mort_val.parse::<f64>().unwrap_or(0.0);
        let cull = cull_val.parse::<f64>().unwrap_or(0.0);

        if is_first_entry() {
            let initial = form_flock_size().parse::<f64>().unwrap_or(0.0);
            let active = (initial - mort - cull).max(0.0);
            if active > 0.0 {
                let y = (eggs / active) * 100.0;
                form_yield.set(format!("{:.1}", y));
            } else {
                form_yield.set("0.0".to_string());
            }
        } else {
            let active_flock = (form_opening_flock() - mort - cull).max(0.0);
            form_flock_size.set(format!("{:.0}", active_flock));
            if active_flock > 0.0 {
                let y = (eggs / active_flock) * 100.0;
                form_yield.set(format!("{:.1}", y));
            } else {
                form_yield.set("0.0".to_string());
            }
        }
    };

    // Auto-sync Total Trays -> Total Eggs + Auto-calc Yield + Auto-suggest Stock
    let mut handle_trays_change = move |val: String| {
        form_trays.set(val.clone());
        let trays = val.parse::<f64>().unwrap_or(0.0);
        let eggs = (trays * 30.0).round() as i32;
        form_good.set(eggs.to_string());

        recalc_flock_and_yield(&val, &form_mortality(), &form_cull());

        let new_stock = form_prev_stock() + trays;
        if (new_stock.fract()).abs() < 0.001 {
            form_stock.set(format!("{:.0}", new_stock));
        } else {
            form_stock.set(format!("{:.1}", new_stock));
        }
    };

    // Auto-sync Total Eggs -> Total Trays + Auto-calc Yield + Auto-suggest Stock
    let mut handle_eggs_change = move |val: String| {
        form_good.set(val.clone());
        let eggs = val.parse::<i32>().unwrap_or(0);
        let trays = (eggs as f64) / 30.0;
        let trays_str = if (trays.fract()).abs() < 0.001 {
            format!("{:.0}", trays)
        } else {
            format!("{:.1}", trays)
        };
        form_trays.set(trays_str.clone());

        recalc_flock_and_yield(&trays_str, &form_mortality(), &form_cull());

        let new_stock = form_prev_stock() + trays;
        if (new_stock.fract()).abs() < 0.001 {
            form_stock.set(format!("{:.0}", new_stock));
        } else {
            form_stock.set(format!("{:.1}", new_stock));
        }
    };

    // When user types the starting flock size for a new batch
    let mut handle_flock_change = move |val: String| {
        form_flock_size.set(val.clone());
        let initial = val.parse::<f64>().unwrap_or(0.0);
        form_opening_flock.set(initial);

        let mort = form_mortality().parse::<f64>().unwrap_or(0.0);
        let cull = form_cull().parse::<f64>().unwrap_or(0.0);
        let active = (initial - mort - cull).max(0.0);

        let trays = form_trays().parse::<f64>().unwrap_or(0.0);
        let eggs = (trays * 30.0).round();
        if active > 0.0 {
            let y = (eggs / active) * 100.0;
            form_yield.set(format!("{:.1}", y));
        } else {
            form_yield.set("0.0".to_string());
        }
    };

    // When mortality is updated -> deduct from live flock and recalculate yield
    let mut handle_mortality_change = move |val: String| {
        form_mortality.set(val.clone());
        recalc_flock_and_yield(&form_trays(), &val, &form_cull());
    };

    // When cull count is updated -> deduct from live flock and recalculate yield
    let mut handle_cull_change = move |val: String| {
        form_cull.set(val.clone());
        recalc_flock_and_yield(&form_trays(), &form_mortality(), &val);
    };

    // When shed / batch dropdown changes in the form
    let mut handle_shed_change = move |new_shed: String| {
        form_shed.set(new_shed.clone());

        let first_time = is_first_entry_for_shed(&new_shed, form_id().as_deref());
        is_first_entry.set(first_time);

        if first_time {
            form_opening_flock.set(0.0);
            form_flock_size.set(String::new());
            form_yield.set("0.0".to_string());
        } else {
            let opening = get_opening_flock_for_shed(&new_shed, form_id().as_deref());
            form_opening_flock.set(opening);
            let mort = form_mortality().parse::<f64>().unwrap_or(0.0);
            let cull = form_cull().parse::<f64>().unwrap_or(0.0);
            let active = (opening - mort - cull).max(0.0);
            form_flock_size.set(format!("{:.0}", active));
        }

        let prev = get_latest_stock_for_shed(&new_shed);
        form_prev_stock.set(prev);

        recalc_flock_and_yield(&form_trays(), &form_mortality(), &form_cull());

        let trays = form_trays().parse::<f64>().unwrap_or(0.0);
        let new_stock = prev + trays;
        if (new_stock.fract()).abs() < 0.001 {
            form_stock.set(format!("{:.0}", new_stock));
        } else {
            form_stock.set(format!("{:.1}", new_stock));
        }
    };

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

        let date = form_date().trim().to_string();
        let shed_name = form_shed().trim().to_string();

        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if shed_name.is_empty() {
            form_error.set("Batch/Shed name is required.".to_string());
            return;
        }

        if is_first_entry() {
            let flock_num = form_flock_size().parse::<u32>().unwrap_or(0);
            if flock_num == 0 {
                form_error.set("Please enter the initial flock size (starting bird count) for this batch.".to_string());
                return;
            }
        }

        let is_first = is_first_entry();
        let initial_flock_to_save = form_flock_size().parse::<u32>().unwrap_or(0);
        let shed_for_save = shed_name.clone();

        let total_trays: f64 = form_trays().parse().unwrap_or(0.0);
        let good: i32 = form_good().parse().unwrap_or(0);
        let damaged: f64 = form_damaged().parse().unwrap_or(0.0);
        let dirty1: f64 = form_dirty1().parse().unwrap_or(0.0);
        let dirty2: f64 = form_dirty2().parse().unwrap_or(0.0);
        let yield_pct: f64 = form_yield().parse().unwrap_or(0.0);
        let stock_trays: f64 = form_stock().parse().unwrap_or(0.0);
        let mortality: i32 = form_mortality().parse().unwrap_or(0);
        let cull: i32 = form_cull().parse().unwrap_or(0);
        let feed: f64 = 0.0;

        let notes_val = if form_notes().trim().is_empty() {
            None
        } else {
            Some(form_notes().trim().to_string())
        };

        form_error.set(String::new());
        is_submitting.set(true);

        let is_edit = form_id().is_some();
        let record = DailyProduction {
            id: form_id().unwrap_or_default(),
            date,
            shed_name,
            total_trays,
            egg_count_good: good,
            egg_count_damaged: damaged,
            dirty_cat1: dirty1,
            dirty_cat2: dirty2,
            production_percentage: yield_pct,
            stock_in_trays: stock_trays,
            mortality_count: mortality,
            cull_count: cull,
            feed_consumed_kg: feed,
            flock_size: form_flock_size().parse::<i32>().unwrap_or(0),
            notes: notes_val,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/production/{}", id), &record).await
            } else {
                api.post("/api/production", &record).await
            };

            is_submitting.set(false);

            match res {
                Ok(_) => {
                    if is_first && initial_flock_to_save > 0 {
                        let mut sizes = batch_flock_sizes();
                        sizes.insert(shed_for_save, initial_flock_to_save);
                        batch_flock_sizes.set(sizes);
                    }
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    logs.restart();
                    let desc = if is_edit { "Production record updated successfully." } else { "Production record logged successfully." };
                    toast_api.success(tr("success"), ToastOptions::new().description(desc));
                }
                Err(e) => {
                    form_error.set(e.clone());
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
            }
        });
    };

    let confirm_delete = move |_| {
        if is_deleting() {
            return;
        }
        if let Some(id) = delete_id() {
            is_deleting.set(true);
            spawn(async move {
                let res = api.delete(&format!("/api/production/{}", id)).await;
                if let Ok(_) = res {
                    delete_id.set(None);
                    logs.restart();
                    toast_api.success(tr("success"), ToastOptions::new().description("Production record deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let records_list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let q = search_query().trim().to_lowercase();
    
    // Dynamically derive complete list of active batches (from custom list + database records)
    let mut all_batches = available_batches();
    for r in &records_list {
        if !r.shed_name.trim().is_empty() && !all_batches.contains(&r.shed_name) {
            all_batches.push(r.shed_name.clone());
        }
    }
    all_batches.sort();
    all_batches.dedup();

    // Filter records by Date Range, Batch, and Search query
    let filtered_records: Vec<_> = records_list.into_iter().filter(|r| {
        let matches_date = date_range().matches(&r.date);
        let matches_batch = if let Some(ref b) = selected_batch() {
            r.shed_name.eq_ignore_ascii_case(b)
        } else {
            true
        };
        let matches_search = if q.is_empty() {
            true
        } else {
            r.shed_name.to_lowercase().contains(&q)
                || r.date.contains(&q)
                || r.notes.as_deref().unwrap_or("").to_lowercase().contains(&q)
        };
        matches_date && matches_batch && matches_search
    }).collect();

    // Aggregate summary metrics for filtered records
    let total_trays_sum: f64 = filtered_records.iter().map(|r| if r.total_trays > 0.0 { r.total_trays } else { (r.egg_count_good as f64) / 30.0 }).sum();
    let total_eggs_sum: i32 = filtered_records.iter().map(|r| r.egg_count_good).sum();
    let total_broken_sum: f64 = filtered_records.iter().map(|r| r.egg_count_damaged).sum();
    let total_dirty1_sum: f64 = filtered_records.iter().map(|r| r.dirty_cat1).sum();
    let total_dirty2_sum: f64 = filtered_records.iter().map(|r| r.dirty_cat2).sum();
    
    // For stock in trays: sum latest closing stock of each distinct batch in filtered records
    let mut batch_latest_stocks = std::collections::HashMap::new();
    for r in &filtered_records {
        batch_latest_stocks.entry(r.shed_name.clone()).or_insert(r.stock_in_trays);
    }
    let total_stock_sum: f64 = batch_latest_stocks.values().sum();

    let avg_yield: f64 = if !filtered_records.is_empty() {
        filtered_records.iter().map(|r| r.production_percentage).sum::<f64>() / (filtered_records.len() as f64)
    } else {
        0.0
    };

    let title_str = tr("production");
    let log_btn_str = tr("log-production");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex justify-between items-center gap-2",
                    div { class: "min-w-0",
                        h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "🥚 {title_str}" }
                        p { class: "hidden sm:block text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Daily Batch Production, Dirty/Broken Egg Register & Stock Inventory" }
                    }
                    div { class: "flex items-center gap-1.5 shrink-0",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| is_manage_batches_open.set(true),
                            span { "⚙️" }
                            span { class: "hidden sm:inline", " Manage Batches" }
                        }
                        Button {
                            onclick: move |_| {
                                form_id.set(None);
                                form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                                let first_b = available_batches().first().cloned().unwrap_or_else(|| "Batch A".to_string());
                                form_shed.set(first_b.clone());

                                let first_time = is_first_entry_for_shed(&first_b, None);
                                is_first_entry.set(first_time);

                                if first_time {
                                    form_opening_flock.set(0.0);
                                    form_flock_size.set(String::new());
                                    form_yield.set("0.0".to_string());
                                } else {
                                    let opening = get_opening_flock_for_shed(&first_b, None);
                                    form_opening_flock.set(opening);
                                    form_flock_size.set(format!("{:.0}", opening));
                                }

                                let prev = get_latest_stock_for_shed(&first_b);
                                form_prev_stock.set(prev);
                                if (prev.fract()).abs() < 0.001 {
                                    form_stock.set(format!("{:.0}", prev));
                                } else {
                                    form_stock.set(format!("{:.1}", prev));
                                }

                                form_trays.set("0".to_string());
                                form_good.set("0".to_string());
                                form_damaged.set("0".to_string());
                                form_dirty1.set("0".to_string());
                                form_dirty2.set("0".to_string());
                                form_mortality.set("0".to_string());
                                form_cull.set("0".to_string());
                                form_notes.set(String::new());
                                form_error.set(String::new());
                                is_sheet_open.set(true);
                            },
                            "+ {log_btn_str}"
                        }
                    }
                }

                // Filter controls: Search, Batch Filter, Date Range Filter, Layout Toggle
                div { class: "flex flex-col gap-2",
                    div { class: "flex items-center gap-2",
                        div { class: "flex-1 min-w-0",
                            Input {
                                placeholder: "{search_ph}",
                                value: "{search_query}",
                                oninput: move |e: Event<FormData>| search_query.set(e.value())
                            }
                        }
                        // Batch Filter Select
                        div { class: "relative shrink-0",
                            select {
                                class: "h-9 rounded-md border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 px-2 sm:px-3 text-xs font-medium text-stone-700 dark:text-stone-300 shadow-xs focus:outline-none focus:ring-2 focus:ring-amber-500 cursor-pointer max-w-[110px] sm:max-w-none",
                                value: selected_batch().unwrap_or_default(),
                                onchange: move |e: Event<FormData>| {
                                    let val = e.value();
                                    if val.is_empty() {
                                        selected_batch.set(None);
                                    } else {
                                        selected_batch.set(Some(val));
                                    }
                                },
                                option { value: "", "All Batches" }
                                for b in all_batches.iter() {
                                    option { key: "{b}", value: "{b}", "{b}" }
                                }
                            }
                        }
                        LayoutToggle {
                            selected: view_layout(),
                            onchange: move |vl| view_layout.set(vl)
                        }
                    }
                    DateRangeFilter {
                        selected: date_range(),
                        onchange: move |dr| date_range.set(dr)
                    }
                }
            }

            // SECTION 2: SCROLLABLE CONTENT — Register Table or Cards
            div { class: "flex-1 overflow-y-auto min-h-0 p-3 md:p-6",
                div { class: "flex flex-col gap-4 md:gap-6 w-full max-w-6xl mx-auto pb-16 md:pb-20",

                    // Production Register Table or Cards
                    match logs.cloned() {
                        Some(Ok(_)) if !filtered_records.is_empty() && view_layout() == ViewLayout::Table => rsx! {
                            Card {
                                div { class: "overflow-x-auto",
                                    table { class: "w-full text-sm text-left border-collapse",
                                        thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                            tr {
                                                th { class: "px-4 py-3", "Date" }
                                                th { class: "px-4 py-3", "Batch / Shed" }
                                                th { class: "px-4 py-3 text-right", "Total Trays" }
                                                th { class: "px-4 py-3 text-right", "Total Eggs" }
                                                th { class: "px-4 py-3 text-right", "Broken Eggs" }
                                                th { class: "px-4 py-3 text-center", "Dirty (Cat 1 | Cat 2)" }
                                                th { class: "px-4 py-3 text-right", "Yield %" }
                                                th { class: "px-4 py-3 text-right", "Stock (Trays)" }
                                                th { class: "px-4 py-3 text-right", "Actions" }
                                            }
                                        }
                                        tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                            for item in filtered_records.iter() {
                                                tr {
                                                    key: "{item.id}",
                                                    class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                    td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{item.date}" }
                                                    td { class: "px-4 py-3 font-semibold text-amber-600 dark:text-amber-400 whitespace-nowrap", "{item.shed_name}" }
                                                    td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300",
                                                        if item.total_trays > 0.0 { "{item.total_trays:.1}" } else { "{(item.egg_count_good as f64 / 30.0):.1}" }
                                                    }
                                                    td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400", "{item.egg_count_good}" }
                                                    td { class: "px-4 py-3 text-right font-medium text-red-600 dark:text-red-400", "{item.egg_count_damaged:.1}" }
                                                    td { class: "px-4 py-3 text-center font-medium text-amber-600 dark:text-amber-300", "{item.dirty_cat1:.1} | {item.dirty_cat2:.1}" }
                                                    td { class: "px-4 py-3 text-right font-medium text-emerald-600 dark:text-emerald-400",
                                                        if item.production_percentage > 0.0 { "{item.production_percentage:.1}%" } else { "-" }
                                                    }
                                                    td { class: "px-4 py-3 text-right font-medium text-blue-600 dark:text-blue-400", "{item.stock_in_trays:.1}" }
                                                    td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                        div { class: "flex items-center justify-end gap-2",
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                onclick: {
                                                                    let edit_item = item.clone();
                                                                    move |_| {
                                                                        form_id.set(Some(edit_item.id.clone()));
                                                                        form_date.set(edit_item.date.clone());
                                                                        form_shed.set(edit_item.shed_name.clone());

                                                                        let edit_shed = edit_item.shed_name.clone();
                                                                        let first_time = is_first_entry_for_shed(&edit_shed, Some(&edit_item.id));
                                                                        is_first_entry.set(first_time);

                                                                        let opening = get_opening_flock_for_shed(&edit_shed, Some(&edit_item.id));
                                                                        form_opening_flock.set(opening);
                                                                        let active = (opening - edit_item.mortality_count as f64 - edit_item.cull_count as f64).max(0.0);
                                                                        form_flock_size.set(format!("{:.0}", active));

                                                                        let prev = (edit_item.stock_in_trays - edit_item.total_trays).max(0.0);
                                                                        form_prev_stock.set(prev);
                                                                        form_trays.set(edit_item.total_trays.to_string());
                                                                        form_good.set(edit_item.egg_count_good.to_string());
                                                                        form_damaged.set(edit_item.egg_count_damaged.to_string());
                                                                        form_dirty1.set(edit_item.dirty_cat1.to_string());
                                                                        form_dirty2.set(edit_item.dirty_cat2.to_string());
                                                                        form_yield.set(edit_item.production_percentage.to_string());
                                                                        form_stock.set(edit_item.stock_in_trays.to_string());
                                                                        form_mortality.set(edit_item.mortality_count.to_string());
                                                                        form_cull.set(edit_item.cull_count.to_string());
                                                                        form_notes.set(edit_item.notes.clone().unwrap_or_default());
                                                                        is_sheet_open.set(true);
                                                                    }
                                                                },
                                                                "{edit_str}"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: {
                                                                    let del_id = item.id.clone();
                                                                    move |_| {
                                                                        delete_id.set(Some(del_id.clone()));
                                                                    }
                                                                },
                                                                "{delete_str}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Daily Register Total Aggregated Summary Row
                                        tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                            tr {
                                                td { class: "px-4 py-3", "TOTAL" }
                                                td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_records.len()} records" }
                                                td { class: "px-4 py-3 text-right text-amber-700 dark:text-amber-300", "{total_trays_sum:.1}" }
                                                td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "{total_eggs_sum}" }
                                                td { class: "px-4 py-3 text-right text-red-700 dark:text-red-300", "{total_broken_sum:.1}" }
                                                td { class: "px-4 py-3 text-center text-amber-700 dark:text-amber-300", "{total_dirty1_sum:.1} | {total_dirty2_sum:.1}" }
                                                td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "{avg_yield:.1}%" }
                                                td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "{total_stock_sum:.1}" }
                                                td { class: "px-4 py-3", "" }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Ok(_)) if !filtered_records.is_empty() => rsx! {
                            div { class: "flex flex-col gap-4",
                                for item in filtered_records.iter() {
                                    Card { key: "{item.id}", class: "border-l-4 border-l-amber-500 dark:border-l-amber-400",
                                        div { class: "p-4 sm:p-5 flex flex-col gap-3",
                                            div { class: "flex justify-between items-center",
                                                div { class: "flex items-center gap-2",
                                                    span { class: "text-sm font-semibold text-stone-900 dark:text-stone-100", "🗓️ {item.date}" }
                                                    span { class: "px-2.5 py-0.5 rounded-full text-xs font-bold bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300 border border-amber-200 dark:border-amber-800",
                                                        "🏷️ {item.shed_name}"
                                                    }
                                                }
                                                if item.production_percentage > 0.0 {
                                                    span { class: "px-2.5 py-0.5 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800",
                                                        "Yield: {item.production_percentage:.1}%"
                                                    }
                                                }
                                            }
                                            div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5 py-2 bg-stone-50 dark:bg-stone-800/50 rounded-lg p-3 border border-stone-200/60 dark:border-stone-700/60 text-xs",
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Total Trays" }
                                                    p { class: "text-base font-bold text-amber-600 dark:text-amber-400 mt-0.5",
                                                        if item.total_trays > 0.0 { "{item.total_trays:.1}" } else { "{(item.egg_count_good as f64 / 30.0):.1}" }
                                                    }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Good Eggs" }
                                                    p { class: "text-base font-bold text-emerald-600 dark:text-emerald-400 mt-0.5", "{item.egg_count_good}" }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Broken / Damaged" }
                                                    p { class: "text-base font-bold text-rose-600 dark:text-rose-400 mt-0.5", "{item.egg_count_damaged:.1}" }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Stock in Trays" }
                                                    p { class: "text-base font-bold text-blue-600 dark:text-blue-400 mt-0.5", "{item.stock_in_trays:.1}" }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Dirty (Cat 1 | 2)" }
                                                    p { class: "text-sm font-semibold text-stone-700 dark:text-stone-300 mt-0.5", "{item.dirty_cat1:.1} | {item.dirty_cat2:.1}" }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Laying Yield" }
                                                    p { class: "text-sm font-semibold text-emerald-600 dark:text-emerald-400 mt-0.5",
                                                        if item.production_percentage > 0.0 { "{item.production_percentage:.1}%" } else { "-" }
                                                    }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Mortality / Cull" }
                                                    p { class: "text-sm font-semibold text-stone-700 dark:text-stone-300 mt-0.5", "{item.mortality_count} / {item.cull_count}" }
                                                }
                                                div {
                                                    p { class: "text-stone-500 dark:text-stone-400 font-medium", "Status" }
                                                    p { class: "text-sm font-bold text-emerald-600 dark:text-emerald-400 mt-0.5", "Logged" }
                                                }
                                            }
                                            if let Some(notes) = &item.notes {
                                                if !notes.is_empty() {
                                                    p { class: "text-xs text-stone-500 dark:text-stone-400 italic bg-stone-100/50 dark:bg-stone-800/30 p-2 rounded", "Note: {notes}" }
                                                }
                                            }
                                            div { class: "flex justify-end gap-2 pt-1 border-t border-stone-100 dark:border-stone-800",
                                                {
                                                    let edit_item = item.clone();
                                                    let del_id = item.id.clone();
                                                    rsx! {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                form_id.set(Some(edit_item.id.clone()));
                                                                form_date.set(edit_item.date.clone());
                                                                form_shed.set(edit_item.shed_name.clone());

                                                                let edit_shed = edit_item.shed_name.clone();
                                                                let first_time = is_first_entry_for_shed(&edit_shed, Some(&edit_item.id));
                                                                is_first_entry.set(first_time);

                                                                let opening = get_opening_flock_for_shed(&edit_shed, Some(&edit_item.id));
                                                                form_opening_flock.set(opening);
                                                                let active = (opening - edit_item.mortality_count as f64 - edit_item.cull_count as f64).max(0.0);
                                                                form_flock_size.set(format!("{:.0}", active));

                                                                let prev = (edit_item.stock_in_trays - edit_item.total_trays).max(0.0);
                                                                form_prev_stock.set(prev);
                                                                form_trays.set(edit_item.total_trays.to_string());
                                                                form_good.set(edit_item.egg_count_good.to_string());
                                                                form_damaged.set(edit_item.egg_count_damaged.to_string());
                                                                form_dirty1.set(edit_item.dirty_cat1.to_string());
                                                                form_dirty2.set(edit_item.dirty_cat2.to_string());
                                                                form_yield.set(edit_item.production_percentage.to_string());
                                                                form_stock.set(edit_item.stock_in_trays.to_string());
                                                                form_mortality.set(edit_item.mortality_count.to_string());
                                                                form_cull.set(edit_item.cull_count.to_string());
                                                                form_notes.set(edit_item.notes.clone().unwrap_or_default());
                                                                is_sheet_open.set(true);
                                                            },
                                                            "{edit_str}"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                delete_id.set(Some(del_id.clone()));
                                                            },
                                                            "{delete_str}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Ok(_)) => rsx! {
                            EmptyState {
                                icon: "🥚",
                                title: "No production records found",
                                description: "No production logs match your search criteria or batch/date filter."
                            }
                        },
                        Some(Err(err)) => rsx! {
                            ErrorState { message: err }
                        },
                        None => rsx! { LoadingState {} }
                    }

                    ConfirmDialog {
                        is_open: delete_id().is_some(),
                        is_deleting: is_deleting(),
                        title: "Delete Production Record".to_string(),
                        description: "Are you sure you want to delete this production record? This action cannot be undone.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| {
                            if !is_deleting() {
                                delete_id.set(None);
                            }
                        }
                    }
                }
            }

            // SECTION 3: LOG PRODUCTION SHEET MODAL
            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle {
                            if form_id().is_some() { "Edit Production Record" } else { "Log Daily Production" }
                        }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[75vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }

                        // Date, Batch Dropdown & Flock Size
                        div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "prod-date", "Date" }
                                Input { r#type: "date", value: "{form_date}", oninput: move |e: Event<FormData>| form_date.set(e.value()) }
                            }
                            div { class: "flex flex-col gap-2",
                                div { class: "flex items-center justify-between",
                                    Label { html_for: "prod-shed", "Batch / Shed" }
                                    button {
                                        class: "text-xs text-amber-600 dark:text-amber-400 hover:underline font-medium",
                                        onclick: move |_| is_manage_batches_open.set(true),
                                        "+ Manage"
                                    }
                                }
                                select {
                                    class: "flex h-10 w-full rounded-md border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-900 px-3 py-2 text-sm text-stone-900 dark:text-stone-100 shadow-sm focus:outline-none focus:ring-2 focus:ring-amber-500",
                                    value: form_shed(),
                                    onchange: move |e: Event<FormData>| handle_shed_change(e.value()),
                                    for b in all_batches.iter() {
                                        option { key: "{b}", value: "{b}", "{b}" }
                                    }
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                div { class: "flex items-center justify-between",
                                    Label { html_for: "prod-flock",
                                        if is_first_entry() { "Initial Flock Size (Birds)" } else { "Live Birds (Flock)" }
                                    }
                                    if is_first_entry() {
                                        span { class: "text-[11px] text-amber-600 dark:text-amber-400 font-semibold", "✏️ First entry: Enter count" }
                                    } else {
                                        span { class: "text-[11px] text-emerald-600 dark:text-emerald-400 font-semibold", "⚡ Auto-calculated" }
                                    }
                                }
                                Input {
                                    r#type: "number",
                                    disabled: !is_first_entry(),
                                    placeholder: if is_first_entry() { "e.g. 10000" } else { "" },
                                    value: "{form_flock_size}",
                                    oninput: move |e: Event<FormData>| handle_flock_change(e.value())
                                }
                                span { class: "text-[11px] text-stone-500 dark:text-stone-400",
                                    if is_first_entry() {
                                        "First log for {form_shed}: set starting flock count"
                                    } else {
                                        "Opening ({form_opening_flock:.0}) - Loss today ({form_mortality}+{form_cull})"
                                    }
                                }
                            }
                        }

                        // Collection: Total Trays & Total Eggs (Dual 2-Way Converter)
                        div { class: "p-3 rounded-lg border border-amber-200/80 dark:border-amber-950/60 bg-amber-50/40 dark:bg-amber-950/20 flex flex-col gap-3",
                            span { class: "text-xs font-semibold uppercase text-amber-800 dark:text-amber-300 tracking-wider", "Egg Collection (1 Tray = 30 Eggs)" }
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-trays", "Total Trays" }
                                    Input {
                                        r#type: "number",
                                        step: "0.1",
                                        value: "{form_trays}",
                                        oninput: move |e: Event<FormData>| handle_trays_change(e.value())
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-good", "Total Eggs" }
                                    Input {
                                        r#type: "number",
                                        value: "{form_good}",
                                        oninput: move |e: Event<FormData>| handle_eggs_change(e.value())
                                    }
                                }
                            }
                        }

                        // Quality & Defects: Broken & Dirty Eggs
                        div { class: "p-3 rounded-lg border border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-900/50 flex flex-col gap-3",
                            span { class: "text-xs font-semibold uppercase text-stone-600 dark:text-stone-400 tracking-wider", "Defects & Quality (Broken / Dirty)" }
                            div { class: "grid grid-cols-3 gap-3",
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-damaged", "Broken Eggs" }
                                    Input {
                                        r#type: "number",
                                        step: "0.5",
                                        value: "{form_damaged}",
                                        oninput: move |e: Event<FormData>| form_damaged.set(e.value())
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-dirty1", "Dirty (Cat 1 / அ)" }
                                    Input {
                                        r#type: "number",
                                        step: "0.5",
                                        value: "{form_dirty1}",
                                        oninput: move |e: Event<FormData>| form_dirty1.set(e.value())
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-dirty2", "Dirty (Cat 2 / வெ)" }
                                    Input {
                                        r#type: "number",
                                        step: "0.5",
                                        value: "{form_dirty2}",
                                        oninput: move |e: Event<FormData>| form_dirty2.set(e.value())
                                    }
                                }
                            }
                        }

                        // Yield % & Stock in Trays (Auto-Calculated)
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-1.5",
                                div { class: "flex items-center justify-between",
                                    Label { html_for: "prod-yield", "Laying Yield (%)" }
                                    span { class: "text-[11px] text-amber-600 dark:text-amber-400 font-semibold", "⚡ Auto-calculated" }
                                }
                                Input {
                                    r#type: "number",
                                    step: "0.1",
                                    placeholder: "e.g. 88.0",
                                    value: "{form_yield}",
                                    oninput: move |e: Event<FormData>| form_yield.set(e.value())
                                }
                                span { class: "text-[11px] text-stone-500 dark:text-stone-400",
                                    "Eggs ÷ {form_flock_size} birds in {form_shed}"
                                }
                            }
                            div { class: "flex flex-col gap-1.5",
                                div { class: "flex items-center justify-between",
                                    Label { html_for: "prod-stock", "Stock in Trays" }
                                    span { class: "text-[11px] text-blue-600 dark:text-blue-400 font-semibold", "📦 Auto-calculated" }
                                }
                                Input {
                                    r#type: "number",
                                    step: "0.5",
                                    disabled: true,
                                    class: "bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-300 cursor-not-allowed",
                                    placeholder: "e.g. 355",
                                    value: "{form_stock}",
                                    oninput: move |e: Event<FormData>| form_stock.set(e.value())
                                }
                                span { class: "text-[11px] text-stone-500 dark:text-stone-400",
                                    "Prev stock ({form_prev_stock:.1}) + Today's trays"
                                }
                            }
                        }

                        // Flock Health (Mortality & Culls)
                        div { class: "p-3 rounded-lg border border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-900/50 flex flex-col gap-3",
                            span { class: "text-xs font-semibold uppercase text-stone-600 dark:text-stone-400 tracking-wider", "Flock Health (Mortality & Culls)" }
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-mortality", "Mortality" }
                                    Input {
                                        r#type: "number",
                                        value: "{form_mortality}",
                                        oninput: move |e: Event<FormData>| handle_mortality_change(e.value())
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "prod-cull", "Cull Count" }
                                    Input {
                                        r#type: "number",
                                        value: "{form_cull}",
                                        oninput: move |e: Event<FormData>| handle_cull_change(e.value())
                                    }
                                }
                            }
                        }

                        div { class: "flex flex-col gap-2",
                            Label { html_for: "prod-notes", "Notes (Optional)" }
                            Input { placeholder: "Vaccination, temperature, adjustments...", value: "{form_notes}", oninput: move |e: Event<FormData>| form_notes.set(e.value()) }
                        }
                    }
                    SheetFooter {
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: is_submitting(),
                            onclick: move |_| is_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            disabled: is_submitting(),
                            loading: is_submitting(),
                            onclick: submit_handler,
                            if form_id().is_some() { "Update Record" } else { "Save Record" }
                        }
                    }
                }
            }

            // SECTION 4: MANAGE BATCHES MODAL
            if is_manage_batches_open() {
                div { class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4",
                    div { class: "bg-white dark:bg-stone-900 rounded-xl shadow-xl border border-stone-200 dark:border-stone-800 w-full max-w-md p-6 flex flex-col gap-4",
                        div { class: "flex items-center justify-between border-b border-stone-200 dark:border-stone-800 pb-3",
                            h3 { class: "text-lg font-bold text-stone-900 dark:text-stone-100", "⚙️ Manage Farm Batches" }
                            button {
                                class: "text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 text-lg font-bold",
                                onclick: move |_| is_manage_batches_open.set(false),
                                "✕"
                            }
                        }

                        // Add new batch form
                        div { class: "flex items-center gap-2",
                            Input {
                                placeholder: "Enter batch name (e.g. Batch G, Shed 2)",
                                value: "{new_batch_name_input}",
                                oninput: move |e: Event<FormData>| new_batch_name_input.set(e.value())
                            }
                            Button {
                                onclick: move |_| {
                                    let name = new_batch_name_input().trim().to_string();
                                    if !name.is_empty() && !available_batches().contains(&name) {
                                        let mut list = available_batches();
                                        list.push(name.clone());
                                        available_batches.set(list);
                                        new_batch_name_input.set(String::new());
                                    }
                                },
                                "+ Add"
                            }
                        }

                        // List of active batches
                        div { class: "flex flex-col gap-2 max-h-60 overflow-y-auto divide-y divide-stone-100 dark:divide-stone-800",
                            for b in available_batches() {
                                div {
                                    key: "{b}",
                                    class: "flex items-center justify-between py-2",
                                    div { class: "flex items-center gap-2",
                                        span { class: "font-medium text-stone-800 dark:text-stone-200 text-sm", "{b}" }
                                        if let Some(flock) = get_initial_flock_for_shed(&b) {
                                            span { class: "text-xs font-semibold text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/40 px-2 py-0.5 rounded-full border border-emerald-200/60 dark:border-emerald-800/60",
                                                "Initial: {flock} birds"
                                            }
                                        } else {
                                            span { class: "text-xs text-stone-400 dark:text-stone-500 bg-stone-100 dark:bg-stone-800 px-2 py-0.5 rounded-full",
                                                "Pending first log"
                                            }
                                        }
                                    }
                                    button {
                                        class: "text-red-500 hover:text-red-700 text-xs font-semibold px-2 py-1 rounded hover:bg-red-50 dark:hover:bg-red-950/30",
                                        onclick: {
                                            let batch_name = b.clone();
                                            move |_| {
                                                let mut list = available_batches();
                                                list.retain(|x| x != &batch_name);
                                                available_batches.set(list);
                                            }
                                        },
                                        "Remove"
                                    }
                                }
                            }
                        }

                        div { class: "flex justify-end pt-2 border-t border-stone-200 dark:border-stone-800",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| is_manage_batches_open.set(false),
                                "Done"
                            }
                        }
                    }
                }
            }
        }
    }
}
