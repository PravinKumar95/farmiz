use dioxus::prelude::*;
use chrono::Utc;
use crate::components::toast::{use_toast, ToastOptions};
use crate::i18n::tr;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::date_range_filter::{DateRange, DateRangeFilter};
use crate::components::layout_toggle::{LayoutToggle, ViewLayout};
use crate::components::party_select::PartySelect;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::{BrokenEggSale, EggSale};
use crate::services::*;

#[component]
pub fn Sales() -> Element {
    let mut active_tab = use_signal(|| Some("standard".to_string()));
    let mut standard_sales = use_egg_sales();
    let mut broken_sales = use_broken_egg_sales();
    let mut prod_logs = use_daily_production();
    let parties_res = use_parties();
    let api = use_auth();
    let toast_api = use_toast();
    let mut is_sheet_open = use_signal(|| false);

    let mut view_layout = use_signal(ViewLayout::default);
    let mut date_range = use_signal(DateRange::default);
    let mut search_query = use_signal(String::new);

    let mut delete_info = use_signal(|| Option::<(String, String)>::None);

    // Form Common State
    let mut form_id = use_signal(|| Option::<String>::None);
    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_party_name = use_signal(String::new);
    let mut form_party_id = use_signal(|| Option::<String>::None);

    // Form Standard Egg Sales State (Trays is primary)
    let mut form_trays = use_signal(String::new);
    let mut form_boxes = use_signal(String::new);
    let mut form_eggs = use_signal(String::new);
    let mut form_size = use_signal(|| "Mixed".to_string());
    let mut form_gross_rate = use_signal(String::new);
    let mut form_discount = use_signal(String::new);
    let mut form_received = use_signal(String::new);
    let mut form_payment_mode = use_signal(|| "Cash".to_string());

    // Form Broken Egg Sales State
    let mut form_broken_trays = use_signal(String::new);
    let mut form_broken_rate = use_signal(String::new);
    let mut form_return_trays = use_signal(|| "0".to_string());
    let mut form_broken_received = use_signal(String::new);
    let mut form_broken_payment_mode = use_signal(|| "Cash".to_string());

    let mut form_error = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Numeric input sanitizer to prevent leading zeroes (e.g. "05" -> "5")
    let sanitize_numeric = |val: &str| -> String {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if trimmed.starts_with("0.") || trimmed == "0" {
            return trimmed.to_string();
        }
        let stripped = trimmed.trim_start_matches('0');
        if stripped.is_empty() {
            "0".to_string()
        } else if stripped.starts_with('.') {
            format!("0{}", stripped)
        } else {
            stripped.to_string()
        }
    };

    // Primary Trays-first Volume Synchronization: 1 Box = 7 Trays = 210 Eggs (30 eggs/tray)
    let mut handle_trays_change = move |val: String| {
        let clean = sanitize_numeric(&val);
        form_trays.set(clean.clone());
        if clean.is_empty() {
            form_boxes.set(String::new());
            form_eggs.set(String::new());
            return;
        }
        let trays = clean.parse::<f64>().unwrap_or(0.0);
        let boxes = trays / 7.0;
        let eggs = (trays * 30.0).round() as i32;
        form_boxes.set(if (boxes.fract()).abs() < 0.001 { format!("{:.0}", boxes) } else { format!("{:.2}", boxes) });
        form_eggs.set(if eggs == 0 { String::new() } else { eggs.to_string() });
    };

    let mut handle_boxes_change = move |val: String| {
        let clean = sanitize_numeric(&val);
        form_boxes.set(clean.clone());
        if clean.is_empty() {
            form_trays.set(String::new());
            form_eggs.set(String::new());
            return;
        }
        let boxes = clean.parse::<f64>().unwrap_or(0.0);
        let trays = boxes * 7.0;
        let eggs = (boxes * 210.0).round() as i32;
        form_trays.set(if (trays.fract()).abs() < 0.001 { format!("{:.0}", trays) } else { format!("{:.1}", trays) });
        form_eggs.set(if eggs == 0 { String::new() } else { eggs.to_string() });
    };

    let mut handle_eggs_change = move |val: String| {
        let clean = sanitize_numeric(&val);
        form_eggs.set(clean.clone());
        if clean.is_empty() {
            form_trays.set(String::new());
            form_boxes.set(String::new());
            return;
        }
        let eggs = clean.parse::<f64>().unwrap_or(0.0);
        let trays = eggs / 30.0;
        let boxes = eggs / 210.0;
        form_trays.set(if (trays.fract()).abs() < 0.001 { format!("{:.0}", trays) } else { format!("{:.1}", trays) });
        form_boxes.set(if (boxes.fract()).abs() < 0.001 { format!("{:.0}", boxes) } else { format!("{:.2}", boxes) });
    };

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

        let date = form_date().trim().to_string();
        let party = form_party_name().trim().to_string();

        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if party.len() < 2 {
            form_error.set("Party is required.".to_string());
            return;
        }

        let is_edit = form_id().is_some();
        if active_tab() == Some("standard".to_string()) {
            let trays: f64 = match form_trays().parse::<f64>() {
                Ok(t) if t > 0.0 => t,
                _ => {
                    form_error.set("Please enter a valid trays count greater than 0.".to_string());
                    return;
                }
            };
            let boxes: i32 = (trays / 7.0).round() as i32;
            let total_eggs: i32 = (trays * 30.0).round() as i32;

            let gross_rate: f64 = match form_gross_rate().parse() {
                Ok(r) if r > 0.0 => r,
                _ => {
                    form_error.set("Invalid gross rate per egg.".to_string());
                    return;
                }
            };
            let less_discount: f64 = form_discount().parse().unwrap_or(0.0);
            let net_rate = (gross_rate - less_discount).max(0.0);
            let received: f64 = form_received().parse().unwrap_or(0.0);
            let total_amount = (total_eggs as f64) * net_rate;
            let balance = total_amount - received;
            let size = form_size();
            let payment_mode = form_payment_mode();

            form_error.set(String::new());
            is_submitting.set(true);

            let new_sale = EggSale {
                id: form_id().unwrap_or_default(),
                date,
                party_name: party,
                party_id: form_party_id(),
                quantity_boxes: boxes,
                total_eggs,
                size,
                gross_rate,
                less_discount,
                net_rate,
                total_amount,
                received_amount: received,
                payment_mode,
                balance,
                user_id: None,
                created_at: None,
            };

            spawn(async move {
                let res = if let Some(id) = form_id() {
                    api.put(&format!("/api/sales/egg/{}", id), &new_sale).await
                } else {
                    api.post("/api/sales/egg", &new_sale).await
                };
                is_submitting.set(false);
                match res {
                    Ok(_) => {
                        is_sheet_open.set(false);
                        form_error.set(String::new());
                        standard_sales.restart();
                        prod_logs.restart();
                        let desc = if is_edit { "Egg sale updated successfully." } else { "Egg sale created successfully." };
                        toast_api.success(tr("success"), ToastOptions::new().description(desc));
                    }
                    Err(e) => {
                        form_error.set(e.clone());
                        toast_api.error(tr("error"), ToastOptions::new().description(e));
                    }
                }
            });
        } else {
            let trays: i32 = match form_broken_trays().parse() {
                Ok(b) if b > 0 => b,
                _ => {
                    form_error.set("Invalid trays count.".to_string());
                    return;
                }
            };
            let rate: f64 = match form_broken_rate().parse() {
                Ok(r) if r >= 0.0 => r,
                _ => {
                    form_error.set("Invalid rate per tray.".to_string());
                    return;
                }
            };
            let received: f64 = form_broken_received().parse().unwrap_or(0.0);
            let return_trays: i32 = form_return_trays().parse().unwrap_or(0);
            let empty_balance = (trays - return_trays).max(0);
            let total_amount = (trays as f64) * rate;

            form_error.set(String::new());
            is_submitting.set(true);

            let new_sale = BrokenEggSale {
                id: form_id().unwrap_or_default(),
                date,
                bakery_name: party,
                party_id: form_party_id(),
                trays_sold: trays,
                rate,
                amount: total_amount,
                payment_received: received,
                return_trays,
                empty_trays_balance: empty_balance,
                balance_amount: total_amount - received,
                user_id: None,
                created_at: None,
            };

            spawn(async move {
                let res = if let Some(id) = form_id() {
                    api.put(&format!("/api/sales/broken/{}", id), &new_sale).await
                } else {
                    api.post("/api/sales/broken", &new_sale).await
                };
                is_submitting.set(false);
                match res {
                    Ok(_) => {
                        is_sheet_open.set(false);
                        form_error.set(String::new());
                        broken_sales.restart();
                        prod_logs.restart();
                        let desc = if is_edit { "Broken egg sale updated successfully." } else { "Broken egg sale created successfully." };
                        toast_api.success(tr("success"), ToastOptions::new().description(desc));
                    }
                    Err(e) => {
                        form_error.set(e.clone());
                        toast_api.error(tr("error"), ToastOptions::new().description(e));
                    }
                }
            });
        }
    };

    let confirm_delete = move |_| {
        if is_deleting() {
            return;
        }
        if let Some((id, s_type)) = delete_info() {
            is_deleting.set(true);
            spawn(async move {
                let endpoint = if s_type == "standard" {
                    format!("/api/sales/egg/{}", id)
                } else {
                    format!("/api/sales/broken/{}", id)
                };
                let res = api.delete(&endpoint).await;
                if let Ok(_) = res {
                    delete_info.set(None);
                    standard_sales.restart();
                    broken_sales.restart();
                    prod_logs.restart();
                    toast_api.success(tr("success"), ToastOptions::new().description("Sale record deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let q = search_query().trim().to_lowercase();

    let all_std_sales = standard_sales.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let all_broken_sales = broken_sales.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let all_prod_records = prod_logs.cloned().and_then(|r| r.ok()).unwrap_or_default();

    // ── LIVE PRODUCTION LOG INVENTORY SYNCHRONIZATION ──
    // 1. Total good egg trays produced across all batches
    let total_produced_good_trays: f64 = all_prod_records.iter().map(|p| {
        if p.total_trays > 0.0 {
            p.total_trays
        } else {
            (p.egg_count_good as f64) / 30.0
        }
    }).sum();
    // 2. Total standard eggs sold in trays
    let total_sold_std_trays: f64 = all_std_sales.iter().map(|s| (s.total_eggs as f64) / 30.0).sum();
    // 3. Live net good egg stock in store
    let available_good_trays = (total_produced_good_trays - total_sold_std_trays).max(0.0);
    let _available_good_boxes = (available_good_trays / 7.0).floor() as i32;

    // 4. Total broken egg trays produced
    let total_produced_broken_trays: f64 = all_prod_records.iter().map(|p| p.egg_count_damaged).sum();
    // 5. Total broken egg trays sold
    let total_sold_broken_trays: f64 = all_broken_sales.iter().map(|s| s.trays_sold as f64).sum();
    // 6. Live net broken egg stock in store
    let available_broken_trays = (total_produced_broken_trays - total_sold_broken_trays).max(0.0);

    // Filter standard sales
    let filtered_std: Vec<_> = all_std_sales.clone().into_iter().filter(|s| {
        let matches_date = date_range().matches(&s.date);
        let matches_search = if q.is_empty() {
            true
        } else {
            s.party_name.to_lowercase().contains(&q) || s.date.contains(&q)
        };
        matches_date && matches_search
    }).collect();

    // Filter broken sales
    let filtered_broken: Vec<_> = all_broken_sales.clone().into_iter().filter(|s| {
        let matches_date = date_range().matches(&s.date);
        let matches_search = if q.is_empty() {
            true
        } else {
            s.bakery_name.to_lowercase().contains(&q) || s.date.contains(&q)
        };
        matches_date && matches_search
    }).collect();

    let total_std_boxes: i32 = filtered_std.iter().map(|s| s.quantity_boxes).sum();
    let total_std_eggs: i32 = filtered_std.iter().map(|s| s.total_eggs).sum();
    let total_std_amount: f64 = filtered_std.iter().map(|s| s.total_amount).sum();
    let total_std_received: f64 = filtered_std.iter().map(|s| s.received_amount).sum();
    let total_std_balance: f64 = filtered_std.iter().map(|s| s.balance).sum();

    let total_broken_trays: i32 = filtered_broken.iter().map(|s| s.trays_sold).sum();
    let total_broken_amount: f64 = filtered_broken.iter().map(|s| s.amount).sum();
    let total_broken_received: f64 = filtered_broken.iter().map(|s| s.payment_received).sum();
    let total_broken_balance: f64 = filtered_broken.iter().map(|s| s.balance_amount).sum();
    let total_empty_trays: i32 = filtered_broken.iter().map(|s| s.empty_trays_balance).sum();

    // Helper: Lookup selected party's ledger balance
    let party_balance_opt = {
        let p_list = parties_res.cloned().and_then(|r| r.ok()).unwrap_or_default();
        let pid = form_party_id();
        let pname = form_party_name();
        p_list.into_iter().find(|p| {
            if let Some(ref id) = pid {
                p.id == *id
            } else if !pname.is_empty() {
                p.name.eq_ignore_ascii_case(&pname)
            } else {
                false
            }
        }).map(|p| p.current_balance)
    };

    let party_balance_info = party_balance_opt.map(|bal| {
        let text = if bal > 0.0 {
            format!("₹{:.2} (Due)", bal)
        } else if bal < 0.0 {
            format!("₹{:.2} (Advance)", bal.abs())
        } else {
            "₹0.00 (Settled)".to_string()
        };
        let class = if bal > 0.0 {
            "font-bold text-amber-600 dark:text-amber-400"
        } else if bal < 0.0 {
            "font-bold text-emerald-600 dark:text-emerald-400"
        } else {
            "font-semibold text-stone-600 dark:text-stone-300"
        };
        (text, class)
    });

    let title_str = tr("sales");
    let add_btn_str = tr("add-sale");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex justify-between items-center gap-2",
                    div { class: "min-w-0",
                        h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "💰 {title_str}" }
                        p { class: "hidden sm:block text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Daily standard egg & broken egg sales dispatch and stock receivables" }
                    }
                    Button {
                        onclick: move |_| {
                            prod_logs.restart();
                            standard_sales.restart();
                            broken_sales.restart();
                            form_id.set(None);
                            form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                            form_party_name.set(String::new());
                            form_party_id.set(None);
                            form_trays.set(String::new());
                            form_boxes.set(String::new());
                            form_eggs.set(String::new());
                            form_size.set("Mixed".to_string());
                            form_gross_rate.set(String::new());
                            form_discount.set(String::new());
                            form_received.set(String::new());
                            form_payment_mode.set("Cash".to_string());
                            form_broken_trays.set(String::new());
                            form_broken_rate.set(String::new());
                            form_return_trays.set("0".to_string());
                            form_broken_received.set(String::new());
                            form_broken_payment_mode.set("Cash".to_string());
                            form_error.set(String::new());
                            is_sheet_open.set(true);
                        },
                        "+ {add_btn_str}"
                    }
                }

                div { class: "flex flex-col gap-2",
                    div { class: "flex items-center gap-2",
                        // Tab Filter Pills (Standard Eggs vs Broken Eggs)
                        div { class: "flex p-0.5 bg-stone-100 dark:bg-stone-800/80 rounded-lg border border-stone-200/80 dark:border-stone-700/60 text-xs font-medium shrink-0",
                            button {
                                class: if active_tab() == Some("standard".to_string()) {
                                    "px-2 sm:px-3 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-xs transition-all text-xs cursor-pointer"
                                } else {
                                    "px-2 sm:px-3 py-1 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all text-xs cursor-pointer"
                                },
                                onclick: move |_| active_tab.set(Some("standard".to_string())),
                                "Standard"
                            }
                            button {
                                class: if active_tab() == Some("broken".to_string()) {
                                    "px-2 sm:px-3 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-xs transition-all text-xs cursor-pointer"
                                } else {
                                    "px-2 sm:px-3 py-1 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all text-xs cursor-pointer"
                                },
                                onclick: move |_| active_tab.set(Some("broken".to_string())),
                                "Broken"
                            }
                        }

                        div { class: "flex-1 min-w-0",
                            Input {
                                placeholder: "{search_ph}",
                                value: "{search_query}",
                                oninput: move |e: Event<FormData>| search_query.set(e.value())
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

            // SECTION 2: SCROLLABLE CONTENT (Table or Cards view)
            div { class: "flex-1 overflow-y-auto min-h-0 p-3 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-16 md:pb-20",

                    if active_tab() == Some("standard".to_string()) {
                        match standard_sales.cloned() {
                            Some(Ok(_)) if !filtered_std.is_empty() => rsx! {
                                if view_layout() == ViewLayout::Table {
                                    // ── STANDARD SALES TABLE VIEW (TRAYS PRIMARY) ──
                                    div { class: "mt-1",
                                        Card {
                                            div { class: "overflow-x-auto",
                                                table { class: "w-full text-sm text-left border-collapse",
                                                    thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                        tr {
                                                            th { class: "px-4 py-3", "Date" }
                                                            th { class: "px-4 py-3", "Customer Party" }
                                                            th { class: "px-4 py-3 text-right", "Trays Sold (Boxes)" }
                                                            th { class: "px-4 py-3 text-center", "Size" }
                                                            th { class: "px-4 py-3 text-right", "Rate (₹/egg)" }
                                                            th { class: "px-4 py-3 text-right", "Total (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Received (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Balance (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Actions" }
                                                        }
                                                    }
                                                    tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                        for sale in filtered_std.iter() {
                                                            {
                                                                let edit_sale = sale.clone();
                                                                let delete_id = sale.id.clone();
                                                                let sale_trays = (sale.total_eggs as f64) / 30.0;
                                                                rsx! {
                                                                    tr {
                                                                        key: "{sale.id}",
                                                                        class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                        td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{sale.date}" }
                                                                        td { class: "px-4 py-3 font-semibold text-blue-600 dark:text-blue-400 whitespace-nowrap", "{sale.party_name}" }
                                                                        td { class: "px-4 py-3 text-right font-medium text-stone-800 dark:text-stone-200 whitespace-nowrap",
                                                                            div { class: "font-bold text-stone-900 dark:text-stone-100", "{sale_trays:.0} Trays" }
                                                                            div { class: "text-[10px] text-stone-400 font-normal",
                                                                                "{sale.quantity_boxes} Boxes • {sale.total_eggs} Eggs"
                                                                            }
                                                                        }
                                                                        td { class: "px-4 py-3 text-center",
                                                                            span { class: "px-2 py-0.5 rounded text-xs font-semibold bg-blue-50 text-blue-700 dark:bg-blue-950/50 dark:text-blue-300 border border-blue-200/60 dark:border-blue-800/60", "{sale.size}" }
                                                                        }
                                                                        td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300",
                                                                            if sale.less_discount > 0.0 {
                                                                                div { class: "font-semibold text-stone-900 dark:text-stone-100", "₹{sale.net_rate:.2}" }
                                                                                div { class: "text-[10px] text-stone-400 line-through", "₹{sale.gross_rate:.2}" }
                                                                            } else {
                                                                                div { "₹{sale.gross_rate:.2}" }
                                                                            }
                                                                        }
                                                                        td { class: "px-4 py-3 text-right font-bold text-stone-900 dark:text-stone-100", "₹{sale.total_amount:.2}" }
                                                                        td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400",
                                                                            div { "₹{sale.received_amount:.2}" }
                                                                            if !sale.payment_mode.is_empty() {
                                                                                div { class: "text-[10px] font-normal text-stone-400", "{sale.payment_mode}" }
                                                                            }
                                                                        }
                                                                        td { class: "px-4 py-3 text-right font-semibold text-rose-600 dark:text-rose-400", "₹{sale.balance:.2}" }
                                                                        td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                                            div { class: "flex items-center justify-end gap-2",
                                                                                Button {
                                                                                    variant: ButtonVariant::Outline,
                                                                                    onclick: move |_| {
                                                                                        form_id.set(Some(edit_sale.id.clone()));
                                                                                        form_date.set(edit_sale.date.clone());
                                                                                        form_party_name.set(edit_sale.party_name.clone());
                                                                                        form_party_id.set(edit_sale.party_id.clone());
                                                                                        let tr_count = (edit_sale.total_eggs as f64) / 30.0;
                                                                                        form_trays.set(if (tr_count.fract()).abs() < 0.001 { format!("{:.0}", tr_count) } else { format!("{:.1}", tr_count) });
                                                                                        form_boxes.set(edit_sale.quantity_boxes.to_string());
                                                                                        form_eggs.set(edit_sale.total_eggs.to_string());
                                                                                        form_size.set(if edit_sale.size.is_empty() { "Mixed".to_string() } else { edit_sale.size.clone() });
                                                                                        let gross = if edit_sale.gross_rate > 0.0 { edit_sale.gross_rate } else { edit_sale.net_rate };
                                                                                        form_gross_rate.set(gross.to_string());
                                                                                        form_discount.set(if edit_sale.less_discount > 0.0 { edit_sale.less_discount.to_string() } else { String::new() });
                                                                                        form_received.set(edit_sale.received_amount.to_string());
                                                                                        form_payment_mode.set(if edit_sale.payment_mode.is_empty() { "Cash".to_string() } else { edit_sale.payment_mode.clone() });
                                                                                        form_error.set(String::new());
                                                                                        is_sheet_open.set(true);
                                                                                    },
                                                                                    "{edit_str}"
                                                                                }
                                                                                Button {
                                                                                    variant: ButtonVariant::Destructive,
                                                                                    onclick: move |_| {
                                                                                        delete_info.set(Some((delete_id.clone(), "standard".to_string())));
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
                                                    tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                                        tr {
                                                            td { class: "px-4 py-3", "TOTAL" }
                                                            td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_std.len()} dispatches" }
                                                            td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{(total_std_eggs as f64 / 30.0):.0} Trays ({total_std_boxes} Bx)" }
                                                            td {}
                                                            td {}
                                                            td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "₹{total_std_amount:.2}" }
                                                            td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "₹{total_std_received:.2}" }
                                                            td { class: "px-4 py-3 text-right text-rose-700 dark:text-rose-300", "₹{total_std_balance:.2}" }
                                                            td {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // ── STANDARD SALES CARD VIEW (TRAYS PRIMARY) ──
                                    div { class: "flex flex-col gap-3 mt-1",
                                        for sale in filtered_std {
                                            Card { key: "{sale.id}",
                                                CardHeader {
                                                    div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                        span { "{sale.date}" }
                                                        div { class: "flex items-center gap-1.5",
                                                            span { class: "px-2 py-0.5 rounded text-[11px] font-semibold bg-blue-50 text-blue-700 dark:bg-blue-950/50 dark:text-blue-300 border border-blue-200/60 dark:border-blue-800/60", "{sale.size}" }
                                                            if !sale.payment_mode.is_empty() {
                                                                span { class: "px-1.5 py-0.5 rounded text-[10px] font-medium bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400", "{sale.payment_mode}" }
                                                            }
                                                        }
                                                    }
                                                }
                                                CardContent {
                                                    div { class: "flex flex-col gap-3",
                                                        div { class: "flex justify-between items-baseline",
                                                            span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{sale.party_name}" }
                                                            span { class: "text-base font-bold text-blue-600 dark:text-blue-400", "₹ {sale.total_amount:.2}" }
                                                        }
                                                        div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                                            div { class: "flex flex-col",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Trays Sold" }
                                                                span { class: "font-bold text-gray-900 dark:text-gray-100", "{(sale.total_eggs as f64 / 30.0):.0} Trays" }
                                                                span { class: "text-[10px] text-gray-400", "{sale.quantity_boxes} Bx ({sale.total_eggs} Eg)" }
                                                            }
                                                            div { class: "flex flex-col",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Received" }
                                                                span { class: "font-semibold text-green-600 dark:text-green-500", "₹{sale.received_amount:.2}" }
                                                                if sale.less_discount > 0.0 {
                                                                    span { class: "text-[10px] text-blue-600 dark:text-blue-400", "Net ₹{sale.net_rate:.2}/egg" }
                                                                } else {
                                                                    span { class: "text-[10px] text-gray-400", "@ ₹{sale.gross_rate:.2}/egg" }
                                                                }
                                                            }
                                                            div { class: "flex flex-col items-end",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Balance" }
                                                                span { class: "font-semibold text-red-500", "₹{sale.balance:.2}" }
                                                            }
                                                        }
                                                    }
                                                }
                                                CardFooter {
                                                    div { class: "flex justify-end gap-2 w-full pt-1",
                                                        {
                                                            let edit_sale = sale.clone();
                                                            let delete_id = sale.id.clone();
                                                            rsx! {
                                                                Button {
                                                                    variant: ButtonVariant::Outline,
                                                                    onclick: move |_| {
                                                                        form_id.set(Some(edit_sale.id.clone()));
                                                                        form_date.set(edit_sale.date.clone());
                                                                        form_party_name.set(edit_sale.party_name.clone());
                                                                        form_party_id.set(edit_sale.party_id.clone());
                                                                        let tr_count = (edit_sale.total_eggs as f64) / 30.0;
                                                                        form_trays.set(if (tr_count.fract()).abs() < 0.001 { format!("{:.0}", tr_count) } else { format!("{:.1}", tr_count) });
                                                                        form_boxes.set(edit_sale.quantity_boxes.to_string());
                                                                        form_eggs.set(edit_sale.total_eggs.to_string());
                                                                        form_size.set(if edit_sale.size.is_empty() { "Mixed".to_string() } else { edit_sale.size.clone() });
                                                                        let gross = if edit_sale.gross_rate > 0.0 { edit_sale.gross_rate } else { edit_sale.net_rate };
                                                                        form_gross_rate.set(gross.to_string());
                                                                        form_discount.set(if edit_sale.less_discount > 0.0 { edit_sale.less_discount.to_string() } else { String::new() });
                                                                        form_received.set(edit_sale.received_amount.to_string());
                                                                        form_payment_mode.set(if edit_sale.payment_mode.is_empty() { "Cash".to_string() } else { edit_sale.payment_mode.clone() });
                                                                        form_error.set(String::new());
                                                                        is_sheet_open.set(true);
                                                                    },
                                                                    "{edit_str}"
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Destructive,
                                                                    onclick: move |_| {
                                                                        delete_info.set(Some((delete_id.clone(), "standard".to_string())));
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
                                }
                            },
                            Some(Ok(_)) => rsx! {
                                EmptyState {
                                    icon: "🥚",
                                    title: "No Standard Egg Sales Found",
                                    description: "No sales records match your search criteria or date filter."
                                }
                            },
                            Some(Err(err)) => rsx! {
                                ErrorState { message: err }
                            },
                            None => rsx! { LoadingState {} }
                        }
                    } else {
                        match broken_sales.cloned() {
                            Some(Ok(_)) if !filtered_broken.is_empty() => rsx! {
                                if view_layout() == ViewLayout::Table {
                                    // ── BROKEN SALES TABLE VIEW ──
                                    div { class: "mt-1",
                                        Card {
                                            div { class: "overflow-x-auto",
                                                table { class: "w-full text-sm text-left border-collapse",
                                                    thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                        tr {
                                                            th { class: "px-4 py-3", "Date" }
                                                            th { class: "px-4 py-3", "Bakery / Customer" }
                                                            th { class: "px-4 py-3 text-right", "Trays Sold" }
                                                            th { class: "px-4 py-3 text-right", "Rate/Tray" }
                                                            th { class: "px-4 py-3 text-right", "Total (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Received (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Balance (₹)" }
                                                            th { class: "px-4 py-3 text-right", "Empty Trays" }
                                                            th { class: "px-4 py-3 text-right", "Actions" }
                                                        }
                                                    }
                                                    tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                        for sale in filtered_broken.iter() {
                                                            {
                                                                let edit_sale = sale.clone();
                                                                let delete_id = sale.id.clone();
                                                                rsx! {
                                                                    tr {
                                                                        key: "{sale.id}",
                                                                        class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                        td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{sale.date}" }
                                                                        td { class: "px-4 py-3 font-semibold text-purple-600 dark:text-purple-400 whitespace-nowrap", "{sale.bakery_name}" }
                                                                        td { class: "px-4 py-3 text-right font-medium text-stone-800 dark:text-stone-200 whitespace-nowrap",
                                                                            div { "{sale.trays_sold} Trays" }
                                                                            div { class: "text-[10px] text-stone-400 font-normal", "{sale.trays_sold * 30} Eggs" }
                                                                        }
                                                                        td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300", "₹{sale.rate:.2}" }
                                                                        td { class: "px-4 py-3 text-right font-bold text-stone-900 dark:text-stone-100", "₹{sale.amount:.2}" }
                                                                        td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400", "₹{sale.payment_received:.2}" }
                                                                        td { class: "px-4 py-3 text-right font-semibold text-rose-600 dark:text-rose-400", "₹{sale.balance_amount:.2}" }
                                                                        td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300",
                                                                            span {
                                                                                class: if sale.empty_trays_balance > 0 { "font-bold text-amber-600 dark:text-amber-400" } else { "text-stone-500" },
                                                                                "{sale.empty_trays_balance} due"
                                                                            }
                                                                            if sale.return_trays > 0 {
                                                                                span { class: "block text-[10px] text-stone-400", "Ret: {sale.return_trays}" }
                                                                            }
                                                                        }
                                                                        td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                                            div { class: "flex items-center justify-end gap-2",
                                                                                Button {
                                                                                    variant: ButtonVariant::Outline,
                                                                                    onclick: move |_| {
                                                                                        form_id.set(Some(edit_sale.id.clone()));
                                                                                        form_date.set(edit_sale.date.clone());
                                                                                        form_party_name.set(edit_sale.bakery_name.clone());
                                                                                        form_party_id.set(edit_sale.party_id.clone());
                                                                                        form_broken_trays.set(edit_sale.trays_sold.to_string());
                                                                                        form_broken_rate.set(edit_sale.rate.to_string());
                                                                                        form_return_trays.set(edit_sale.return_trays.to_string());
                                                                                        form_broken_received.set(edit_sale.payment_received.to_string());
                                                                                        form_broken_payment_mode.set("Cash".to_string());
                                                                                        form_error.set(String::new());
                                                                                        is_sheet_open.set(true);
                                                                                    },
                                                                                    "{edit_str}"
                                                                                }
                                                                                Button {
                                                                                    variant: ButtonVariant::Destructive,
                                                                                    onclick: move |_| {
                                                                                        delete_info.set(Some((delete_id.clone(), "broken".to_string())));
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
                                                    tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                                        tr {
                                                            td { class: "px-4 py-3", "TOTAL" }
                                                            td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_broken.len()} sales" }
                                                            td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{total_broken_trays} Trays" }
                                                            td {}
                                                            td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "₹{total_broken_amount:.2}" }
                                                            td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "₹{total_broken_received:.2}" }
                                                            td { class: "px-4 py-3 text-right text-rose-700 dark:text-rose-300", "₹{total_broken_balance:.2}" }
                                                            td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{total_empty_trays}" }
                                                            td {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // ── BROKEN SALES CARD VIEW ──
                                    div { class: "flex flex-col gap-3 mt-1",
                                        for sale in filtered_broken {
                                            Card { key: "{sale.id}",
                                                CardHeader {
                                                    div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                        span { "{sale.date}" }
                                                        span { "{sale.trays_sold} Trays @ ₹{sale.rate:.2}" }
                                                    }
                                                }
                                                CardContent {
                                                    div { class: "flex flex-col gap-3",
                                                        div { class: "flex justify-between items-baseline",
                                                            span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{sale.bakery_name}" }
                                                            span { class: "text-base font-bold text-blue-600 dark:text-blue-400", "₹ {sale.amount:.2}" }
                                                        }
                                                        div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                                            div { class: "flex flex-col",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Received" }
                                                                span { class: "font-semibold text-green-600 dark:text-green-500", "₹{sale.payment_received:.2}" }
                                                            }
                                                            div { class: "flex flex-col",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Balance" }
                                                                span { class: "font-semibold text-red-500", "₹{sale.balance_amount:.2}" }
                                                            }
                                                            div { class: "flex flex-col items-end",
                                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Empty Trays" }
                                                                span {
                                                                    class: if sale.empty_trays_balance > 0 { "font-bold text-amber-600 dark:text-amber-400" } else { "text-stone-600 dark:text-stone-300" },
                                                                    "{sale.empty_trays_balance} due"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                CardFooter {
                                                    div { class: "flex justify-end gap-2 w-full pt-1",
                                                        {
                                                            let edit_sale = sale.clone();
                                                            let delete_id = sale.id.clone();
                                                            rsx! {
                                                                Button {
                                                                    variant: ButtonVariant::Outline,
                                                                    onclick: move |_| {
                                                                        form_id.set(Some(edit_sale.id.clone()));
                                                                        form_date.set(edit_sale.date.clone());
                                                                        form_party_name.set(edit_sale.bakery_name.clone());
                                                                        form_party_id.set(edit_sale.party_id.clone());
                                                                        form_broken_trays.set(edit_sale.trays_sold.to_string());
                                                                        form_broken_rate.set(edit_sale.rate.to_string());
                                                                        form_return_trays.set(edit_sale.return_trays.to_string());
                                                                        form_broken_received.set(edit_sale.payment_received.to_string());
                                                                        form_broken_payment_mode.set("Cash".to_string());
                                                                        form_error.set(String::new());
                                                                        is_sheet_open.set(true);
                                                                    },
                                                                    "{edit_str}"
                                                                }
                                                                Button {
                                                                    variant: ButtonVariant::Destructive,
                                                                    onclick: move |_| {
                                                                        delete_info.set(Some((delete_id.clone(), "broken".to_string())));
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
                                }
                            },
                            Some(Ok(_)) => rsx! {
                                EmptyState {
                                    icon: "🥚",
                                    title: "No Broken Egg Sales Found",
                                    description: "No broken egg sales records match your search criteria or date filter."
                                }
                            },
                            Some(Err(err)) => rsx! {
                                ErrorState { message: err }
                            },
                            None => rsx! { LoadingState {} }
                        }
                    }

                    ConfirmDialog {
                        is_open: delete_info().is_some(),
                        is_deleting: is_deleting(),
                        title: "Delete Sale Record".to_string(),
                        description: "Are you sure you want to delete this sale record? The party ledger balance and production stock will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| {
                            if !is_deleting() {
                                delete_info.set(None);
                            }
                        }
                    }
                }
            }

            // ── SHEET MODAL FOR ADD / EDIT SALE ──
            if is_sheet_open() {
                {
                    let orig_sale_trays = if let Some(ref edit_id) = form_id() {
                        all_std_sales.iter().find(|s| s.id == *edit_id).map(|s| (s.total_eggs as f64) / 30.0).unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    let effective_good_trays = available_good_trays + orig_sale_trays;
                    let current_trays = form_trays().parse::<f64>().unwrap_or(0.0);
                    let remaining_stock = effective_good_trays - current_trays;
                    let boxes_num = current_trays / 7.0;
                    let eggs_num = current_trays * 30.0;
                    let g_rate = form_gross_rate().parse::<f64>().unwrap_or(0.0);
                    let disc = form_discount().parse::<f64>().unwrap_or(0.0);
                    let net_rate = (g_rate - disc).max(0.0);
                    let est_bill = eggs_num * net_rate;
                    let p_rec = form_received().parse::<f64>().unwrap_or(0.0);
                    let bal_due = est_bill - p_rec;
                    let avail_str = format!("{:.0}", effective_good_trays);
                    let sz = form_size();
                    let p_mode = form_payment_mode();

                    let orig_broken_trays = if let Some(ref edit_id) = form_id() {
                        all_broken_sales.iter().find(|s| s.id == *edit_id).map(|s| s.trays_sold as f64).unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    let effective_broken_trays = available_broken_trays + orig_broken_trays;
                    let b_trays_num = form_broken_trays().parse::<f64>().unwrap_or(0.0);
                    let b_rate_num = form_broken_rate().parse::<f64>().unwrap_or(0.0);
                    let b_ret_trays = form_return_trays().parse::<i32>().unwrap_or(0);
                    let b_empty_due = (b_trays_num as i32 - b_ret_trays).max(0);
                    let b_bill = b_trays_num * b_rate_num;
                    let b_rec = form_broken_received().parse::<f64>().unwrap_or(0.0);
                    let b_bal_due = b_bill - b_rec;
                    let b_remaining = effective_broken_trays - b_trays_num;
                    let broken_avail_str = format!("{:.0}", effective_broken_trays);
                    let b_p_mode = form_broken_payment_mode();

                    rsx! {
                        Sheet {
                            open: Some(is_sheet_open()),
                            on_open_change: move |open| is_sheet_open.set(open),
                            SheetHeader {
                                SheetTitle {
                                    if form_id().is_some() {
                                        if active_tab() == Some("standard".to_string()) {
                                            "Edit Standard Egg Sale"
                                        } else {
                                            "Edit Broken Egg Sale"
                                        }
                                    } else if active_tab() == Some("standard".to_string()) {
                                        "Add Standard Egg Sale"
                                    } else {
                                        "Add Broken Egg Sale"
                                    }
                                }
                            }
                            div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[75vh]",
                                if !form_error().is_empty() {
                                    div { class: "p-3 rounded-lg bg-rose-50 dark:bg-rose-950/40 text-rose-600 dark:text-rose-400 text-xs border border-rose-200 dark:border-rose-900", "{form_error}" }
                                }

                                // Date field
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "sale-date", "Sale Date" }
                                    Input {
                                        id: "sale-date",
                                        r#type: "date",
                                        value: "{form_date}",
                                        oninput: move |e: Event<FormData>| form_date.set(e.value())
                                    }
                                }

                                // Party Selector
                                div { class: "flex flex-col gap-1.5",
                                    Label { html_for: "sale-party", if active_tab() == Some("standard".to_string()) { "Customer Party" } else { "Bakery Party" } }
                                    PartySelect {
                                        value: form_party_id().unwrap_or_else(|| form_party_name()),
                                        filter_type: if active_tab() == Some("standard".to_string()) { Some("CUSTOMER".to_string()) } else { Some("BAKERY".to_string()) },
                                        onchange: move |(pid, pname): (String, String)| {
                                            if !pid.is_empty() {
                                                form_party_id.set(Some(pid));
                                            } else {
                                                form_party_id.set(None);
                                            }
                                            form_party_name.set(pname);
                                        }
                                    }
                                    if let Some((bal_str, bal_cls)) = party_balance_info.clone() {
                                        div { class: "flex items-center justify-between px-2.5 py-1.5 rounded-md bg-stone-100 dark:bg-stone-800 text-xs mt-0.5",
                                            span { class: "text-stone-500 dark:text-stone-400 font-medium", "Current Party Balance:" }
                                            span { class: "{bal_cls}", "{bal_str}" }
                                        }
                                    }
                                }

                                if active_tab() == Some("standard".to_string()) {
                                    // ── STANDARD EGG SALE FORM FIELDS (TRAYS PRIMARY & STOCK SYNC) ──
                                    // 1. Live Production Log Stock Sync Card
                                    div { class: "p-3 rounded-lg bg-emerald-50/70 dark:bg-emerald-950/30 border border-emerald-200/60 dark:border-emerald-800/50 flex flex-col gap-1.5",
                                        div { class: "flex items-center justify-between text-xs",
                                            span { class: "font-semibold text-emerald-900 dark:text-emerald-200 flex items-center gap-1.5",
                                                span { "📦" }
                                                span { "Production Log Available Stock" }
                                            }
                                            div { class: "flex items-center gap-2",
                                                span { class: "font-bold text-sm text-emerald-800 dark:text-emerald-300",
                                                    "{available_good_trays:.1} Trays"
                                                }
                                                if available_good_trays > 0.0 {
                                                    button {
                                                        r#type: "button",
                                                        class: "text-[11px] font-semibold text-emerald-700 dark:text-emerald-400 underline hover:opacity-80 cursor-pointer",
                                                        onclick: move |_| handle_trays_change(avail_str.clone()),
                                                        "Sell All"
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "text-[11px] text-emerald-700/80 dark:text-emerald-400/80 flex justify-between",
                                            span { "Equivalent in Boxes & Eggs:" }
                                            span { "{(effective_good_trays / 7.0):.1} Boxes ({(effective_good_trays * 30.0):.0} Eggs)" }
                                        }
                                        if current_trays > 0.0 {
                                            div { class: "flex items-center justify-between text-xs pt-1 border-t border-emerald-200/60 dark:border-emerald-800/40",
                                                span { class: "text-emerald-800 dark:text-emerald-200 font-medium", "Stock Remaining After Dispatch:" }
                                                span {
                                                    class: if remaining_stock >= 0.0 { "font-bold text-emerald-800 dark:text-emerald-300" } else { "font-bold text-rose-600 dark:text-rose-400" },
                                                    "{remaining_stock:.1} Trays ({(remaining_stock / 7.0):.1} Boxes)"
                                                }
                                            }
                                            if remaining_stock < 0.0 {
                                                div { class: "text-[11px] font-medium text-rose-600 dark:text-rose-400 flex items-center gap-1",
                                                    span { "⚠️" }
                                                    span { "Dispatch exceeds current production stock by {(-remaining_stock):.1} trays!" }
                                                }
                                            }
                                        }
                                    }

                                    // 2. Primary Trays Input (with auto-sync Boxes and Total Eggs)
                                    div { class: "p-3 rounded-lg bg-stone-50 dark:bg-stone-800/50 border border-stone-200/80 dark:border-stone-700/60 flex flex-col gap-2.5",
                                        div { class: "flex items-center justify-between",
                                            Label { html_for: "sale-trays", class: "font-semibold text-stone-900 dark:text-stone-100", "Quantity (Trays — Primary)" }
                                            span { class: "text-[10px] text-stone-400", "1 Tray = 30 Eggs" }
                                        }
                                        Input {
                                            id: "sale-trays",
                                            r#type: "number",
                                            value: "{form_trays}",
                                            oninput: move |e: Event<FormData>| handle_trays_change(e.value()),
                                            placeholder: "e.g. 350 trays"
                                        }

                                        // Secondary auto-synced Boxes and Eggs controls
                                        div { class: "grid grid-cols-2 gap-2 pt-1 border-t border-stone-200/60 dark:border-stone-700/60",
                                            div { class: "flex flex-col gap-1",
                                                Label { html_for: "sale-boxes", class: "text-[11px] text-stone-500 dark:text-stone-400", "Equivalent Boxes (7 Tr/Bx)" }
                                                Input {
                                                    id: "sale-boxes",
                                                    r#type: "number",
                                                    value: "{form_boxes}",
                                                    oninput: move |e: Event<FormData>| handle_boxes_change(e.value()),
                                                    placeholder: "50"
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                Label { html_for: "sale-eggs", class: "text-[11px] text-stone-500 dark:text-stone-400", "Total Eggs" }
                                                Input {
                                                    id: "sale-eggs",
                                                    r#type: "number",
                                                    value: "{form_eggs}",
                                                    oninput: move |e: Event<FormData>| handle_eggs_change(e.value()),
                                                    placeholder: "10500"
                                                }
                                            }
                                        }
                                    }

                                    // 3. Egg Size Selector
                                    div { class: "flex flex-col gap-1.5",
                                        Label { html_for: "sale-size", "Egg Size" }
                                        div { class: "grid grid-cols-5 gap-1.5",
                                            for s in &["Mixed", "Large", "Medium", "Small", "Jumbo"] {
                                                button {
                                                    key: "{s}",
                                                    r#type: "button",
                                                    class: if form_size() == *s {
                                                        "py-1.5 px-1 text-xs font-semibold rounded-md bg-blue-600 text-white shadow-xs text-center transition-all cursor-pointer"
                                                    } else {
                                                        "py-1.5 px-1 text-xs font-medium rounded-md bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 hover:bg-stone-200 dark:hover:bg-stone-700 border border-stone-200/60 dark:border-stone-700/60 text-center transition-all cursor-pointer"
                                                    },
                                                    onclick: {
                                                        let size_val = s.to_string();
                                                        move |_| form_size.set(size_val.clone())
                                                    },
                                                    "{s}"
                                                }
                                            }
                                        }
                                    }

                                    // 4. NECC Gross Rate vs Less Discount (Net Rate)
                                    div { class: "grid grid-cols-2 gap-2.5",
                                        div { class: "flex flex-col gap-1.5",
                                            Label { html_for: "gross-rate", "Gross / NECC Rate (₹)" }
                                            Input {
                                                id: "gross-rate",
                                                r#type: "number",
                                                step: "0.01",
                                                value: "{form_gross_rate}",
                                                oninput: move |e: Event<FormData>| form_gross_rate.set(sanitize_numeric(&e.value())),
                                                placeholder: "e.g. 5.25"
                                            }
                                        }
                                        div { class: "flex flex-col gap-1.5",
                                            Label { html_for: "less-discount", "Less Discount (₹)" }
                                            Input {
                                                id: "less-discount",
                                                r#type: "number",
                                                step: "0.01",
                                                value: "{form_discount}",
                                                oninput: move |e: Event<FormData>| form_discount.set(sanitize_numeric(&e.value())),
                                                placeholder: "e.g. 0.20"
                                            }
                                        }
                                    }

                                    // Net Rate Badge
                                    div { class: "flex items-center justify-between px-3 py-2 rounded-lg bg-blue-50/70 dark:bg-blue-950/30 border border-blue-200/60 dark:border-blue-900/50 text-xs",
                                        div { class: "flex items-center gap-2",
                                            span { class: "text-blue-700 dark:text-blue-300 font-medium", "Net Effective Rate:" }
                                            span { class: "font-bold text-sm text-blue-900 dark:text-blue-100", "₹{net_rate:.2} / egg" }
                                        }
                                        if disc > 0.0 {
                                            span { class: "text-stone-500 dark:text-stone-400 font-normal", "(₹{g_rate:.2} - ₹{disc:.2})" }
                                        }
                                    }

                                    // 5. Payment Mode Selector
                                    div { class: "flex flex-col gap-1.5",
                                        Label { html_for: "sale-payment-mode", "Payment Mode" }
                                        div { class: "grid grid-cols-3 sm:grid-cols-5 gap-1.5",
                                            for mode in &["Cash", "UPI", "Bank Transfer", "Cheque", "Credit"] {
                                                button {
                                                    key: "{mode}",
                                                    r#type: "button",
                                                    class: if form_payment_mode() == *mode {
                                                        "py-1.5 px-1 text-xs font-semibold rounded-md bg-emerald-600 text-white shadow-xs text-center transition-all cursor-pointer"
                                                    } else {
                                                        "py-1.5 px-1 text-xs font-medium rounded-md bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 hover:bg-stone-200 dark:hover:bg-stone-700 border border-stone-200/60 dark:border-stone-700/60 text-center transition-all cursor-pointer"
                                                    },
                                                    onclick: {
                                                        let mode_val = mode.to_string();
                                                        move |_| {
                                                            form_payment_mode.set(mode_val.clone());
                                                            if mode_val == "Credit" && form_received().trim().is_empty() {
                                                                form_received.set("0".to_string());
                                                            }
                                                        }
                                                    },
                                                    "{mode}"
                                                }
                                            }
                                        }
                                    }

                                    // 6. Payment Received Input with quick full-pay button
                                    div { class: "flex flex-col gap-1.5",
                                        div { class: "flex justify-between items-center",
                                            Label { html_for: "sale-received", "Received Payment (₹)" }
                                            if est_bill > 0.0 {
                                                button {
                                                    r#type: "button",
                                                    class: "text-[11px] font-semibold text-blue-600 dark:text-blue-400 hover:underline cursor-pointer",
                                                    onclick: move |_| form_received.set(format!("{:.2}", est_bill)),
                                                    "Fill Full ₹{est_bill:.2}"
                                                }
                                            }
                                        }
                                        Input {
                                            id: "sale-received",
                                            r#type: "number",
                                            step: "0.01",
                                            value: "{form_received}",
                                            oninput: move |e: Event<FormData>| form_received.set(sanitize_numeric(&e.value())),
                                            placeholder: "0.00"
                                        }
                                    }

                                    // 7. Live Calculation Preview Card
                                    div { class: "p-3 rounded-lg bg-blue-50/80 dark:bg-blue-950/40 border border-blue-200/80 dark:border-blue-800/60 text-xs space-y-1.5 mt-1",
                                        div { class: "font-medium text-blue-900 dark:text-blue-200 flex justify-between",
                                            span { "📦 Total Dispatch Volume" }
                                            span { class: "font-bold", "{current_trays:.0} Trays ({boxes_num:.1} Bx = {eggs_num:.0} Eggs) • {sz}" }
                                        }
                                        div { class: "font-medium text-blue-900 dark:text-blue-200 flex justify-between",
                                            span { "💵 Estimated Total Bill" }
                                            span { class: "font-bold text-sm text-blue-700 dark:text-blue-300", "₹ {est_bill:.2}" }
                                        }
                                        div { class: "font-medium text-blue-900 dark:text-blue-200 flex justify-between",
                                            span { "💳 Received via {p_mode}" }
                                            span { class: "font-semibold text-emerald-600 dark:text-emerald-400", "₹ {p_rec:.2}" }
                                        }
                                        div { class: "font-medium flex justify-between pt-1 border-t border-blue-200/60 dark:border-blue-800/40",
                                            span { class: "text-gray-600 dark:text-gray-300", "⏳ Balance Due on this Bill" }
                                            span {
                                                class: if bal_due > 0.0 { "font-bold text-amber-600 dark:text-amber-400" } else { "font-bold text-emerald-600 dark:text-emerald-400" },
                                                "₹ {bal_due:.2}"
                                            }
                                        }
                                    }
                                } else {
                                    // ── BROKEN EGG SALE FORM FIELDS (TRAYS PRIMARY & STOCK SYNC) ──
                                    // Live Broken Egg Stock from Production
                                    div { class: "p-3 rounded-lg bg-purple-50/70 dark:bg-purple-950/30 border border-purple-200/60 dark:border-purple-800/50 flex flex-col gap-1.5",
                                        div { class: "flex items-center justify-between text-xs",
                                            span { class: "font-semibold text-purple-900 dark:text-purple-200 flex items-center gap-1.5",
                                                span { "🍳" }
                                                span { "Production Log Broken Stock" }
                                            }
                                            div { class: "flex items-center gap-2",
                                                span { class: "font-bold text-sm text-purple-800 dark:text-purple-300",
                                                    "{available_broken_trays:.1} Trays"
                                                }
                                                if available_broken_trays > 0.0 {
                                                    button {
                                                        r#type: "button",
                                                        class: "text-[11px] font-semibold text-purple-700 dark:text-purple-400 underline hover:opacity-80 cursor-pointer",
                                                        onclick: move |_| form_broken_trays.set(broken_avail_str.clone()),
                                                        "Sell All"
                                                    }
                                                }
                                            }
                                        }
                                        div { class: "text-[11px] text-purple-700/80 dark:text-purple-400/80 flex justify-between",
                                            span { "Damaged Eggs Count:" }
                                            span { "{available_broken_trays * 30.0:.0} Eggs" }
                                        }
                                        if b_trays_num > 0.0 {
                                            div { class: "flex items-center justify-between text-xs pt-1 border-t border-purple-200/60 dark:border-purple-800/40",
                                                span { class: "text-purple-800 dark:text-purple-200 font-medium", "Remaining Broken Stock:" }
                                                span {
                                                    class: if b_remaining >= 0.0 { "font-bold text-purple-800 dark:text-purple-300" } else { "font-bold text-rose-600 dark:text-rose-400" },
                                                    "{b_remaining:.1} Trays"
                                                }
                                            }
                                            if b_remaining < 0.0 {
                                                div { class: "text-[11px] font-medium text-rose-600 dark:text-rose-400 flex items-center gap-1",
                                                    span { "⚠️" }
                                                    span { "Dispatch exceeds available broken stock by {(-b_remaining):.1} trays!" }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 gap-2.5",
                                        div { class: "flex flex-col gap-1.5",
                                            Label { html_for: "broken-trays", "Quantity (Trays)" }
                                            Input {
                                                id: "broken-trays",
                                                r#type: "number",
                                                value: "{form_broken_trays}",
                                                oninput: move |e: Event<FormData>| form_broken_trays.set(sanitize_numeric(&e.value())),
                                                placeholder: "e.g. 20"
                                            }
                                        }
                                        div { class: "flex flex-col gap-1.5",
                                            Label { html_for: "broken-rate", "Rate per Tray (₹)" }
                                            Input {
                                                id: "broken-rate",
                                                r#type: "number",
                                                step: "0.01",
                                                value: "{form_broken_rate}",
                                                oninput: move |e: Event<FormData>| form_broken_rate.set(sanitize_numeric(&e.value())),
                                                placeholder: "e.g. 130.00"
                                            }
                                        }
                                    }

                                    // Empty Trays tracking
                                    div { class: "p-3 rounded-lg bg-stone-50 dark:bg-stone-800/50 border border-stone-200/80 dark:border-stone-700/60 flex flex-col gap-2",
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-xs font-semibold text-stone-900 dark:text-stone-100", "🍳 Plastic Tray Ledger" }
                                            span { class: "text-[10px] text-stone-400", "Track returned empty trays" }
                                        }
                                        div { class: "flex flex-col gap-1.5",
                                            Label { html_for: "return-trays", "Returned Empty Trays" }
                                            Input {
                                                id: "return-trays",
                                                r#type: "number",
                                                value: "{form_return_trays}",
                                                oninput: move |e: Event<FormData>| form_return_trays.set(sanitize_numeric(&e.value())),
                                                placeholder: "0"
                                            }
                                        }
                                        div { class: "flex items-center justify-between text-xs pt-1 border-t border-stone-200/60 dark:border-stone-700/60",
                                            span { class: "text-stone-600 dark:text-stone-400", "Empty Trays Pending:" }
                                            span {
                                                class: if b_empty_due > 0 { "font-bold text-amber-600 dark:text-amber-400" } else { "font-bold text-emerald-600 dark:text-emerald-400" },
                                                "{b_empty_due} Trays Owed"
                                            }
                                        }
                                    }

                                    // Payment Mode Selector
                                    div { class: "flex flex-col gap-1.5",
                                        Label { html_for: "broken-payment-mode", "Payment Mode" }
                                        div { class: "grid grid-cols-3 sm:grid-cols-5 gap-1.5",
                                            for mode in &["Cash", "UPI", "Bank Transfer", "Cheque", "Credit"] {
                                                button {
                                                    key: "{mode}",
                                                    r#type: "button",
                                                    class: if form_broken_payment_mode() == *mode {
                                                        "py-1.5 px-1 text-xs font-semibold rounded-md bg-emerald-600 text-white shadow-xs text-center transition-all cursor-pointer"
                                                    } else {
                                                        "py-1.5 px-1 text-xs font-medium rounded-md bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 hover:bg-stone-200 dark:hover:bg-stone-700 border border-stone-200/60 dark:border-stone-700/60 text-center transition-all cursor-pointer"
                                                    },
                                                    onclick: {
                                                        let mode_val = mode.to_string();
                                                        move |_| {
                                                            form_broken_payment_mode.set(mode_val.clone());
                                                            if mode_val == "Credit" && form_broken_received().trim().is_empty() {
                                                                form_broken_received.set("0".to_string());
                                                            }
                                                        }
                                                    },
                                                    "{mode}"
                                                }
                                            }
                                        }
                                    }

                                    // Received payment
                                    div { class: "flex flex-col gap-1.5",
                                        div { class: "flex justify-between items-center",
                                            Label { html_for: "broken-received", "Received Payment Amount (₹)" }
                                            if b_bill > 0.0 {
                                                button {
                                                    r#type: "button",
                                                    class: "text-[11px] font-semibold text-blue-600 dark:text-blue-400 hover:underline cursor-pointer",
                                                    onclick: move |_| form_broken_received.set(format!("{:.2}", b_bill)),
                                                    "Fill Full ₹{b_bill:.2}"
                                                }
                                            }
                                        }
                                        Input {
                                            id: "broken-received",
                                            r#type: "number",
                                            step: "0.01",
                                            value: "{form_broken_received}",
                                            oninput: move |e: Event<FormData>| form_broken_received.set(sanitize_numeric(&e.value())),
                                            placeholder: "0.00"
                                        }
                                    }

                                    // Live Calculation Preview Card for Broken
                                    div { class: "p-3 rounded-lg bg-purple-50/80 dark:bg-purple-950/40 border border-purple-200/80 dark:border-purple-800/60 text-xs space-y-1.5 mt-1",
                                        div { class: "font-medium text-purple-900 dark:text-purple-200 flex justify-between",
                                            span { "🍳 Total Trays Volume" }
                                            span { class: "font-bold", "{b_trays_num:.0} Trays ({(b_trays_num * 30.0):.0} Eggs)" }
                                        }
                                        div { class: "font-medium text-purple-900 dark:text-purple-200 flex justify-between",
                                            span { "💵 Estimated Total Bill" }
                                            span { class: "font-bold text-sm text-purple-700 dark:text-purple-300", "₹ {b_bill:.2}" }
                                        }
                                        div { class: "font-medium text-purple-900 dark:text-purple-200 flex justify-between",
                                            span { "💳 Received via {b_p_mode}" }
                                            span { class: "font-semibold text-emerald-600 dark:text-emerald-400", "₹ {b_rec:.2}" }
                                        }
                                        div { class: "font-medium flex justify-between pt-1 border-t border-purple-200/60 dark:border-purple-800/40",
                                            span { class: "text-gray-600 dark:text-gray-300", "⏳ Balance Due on this Bill" }
                                            span {
                                                class: if b_bal_due > 0.0 { "font-bold text-amber-600 dark:text-amber-400" } else { "font-bold text-emerald-600 dark:text-emerald-400" },
                                                "₹ {b_bal_due:.2}"
                                            }
                                        }
                                    }
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
                                    if form_id().is_some() { "Update Sale" } else { "Save Sale" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
