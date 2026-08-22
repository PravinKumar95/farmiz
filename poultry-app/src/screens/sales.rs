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
use crate::components::month_filter::MonthFilter;
use crate::components::party_select::PartySelect;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::{BrokenEggSale, EggSale};
use crate::services::*;

#[component]
pub fn Sales() -> Element {
    let mut active_tab = use_signal(|| Some("standard".to_string()));
    let mut standard_sales = use_egg_sales();
    let mut broken_sales = use_broken_egg_sales();
    let api = use_auth();
    let toast_api = use_toast();
    let mut is_sheet_open = use_signal(|| false);

    let current_month_str = Utc::now().format("%Y-%m").to_string();
    let mut selected_month = use_signal(move || Some(current_month_str.clone()));
    let mut search_query = use_signal(String::new);

    let mut delete_info = use_signal(|| Option::<(String, String)>::None);

    let mut form_id = use_signal(|| Option::<String>::None);
    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_party_name = use_signal(String::new);
    let mut form_party_id = use_signal(|| Option::<String>::None);
    let mut form_boxes = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_received = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

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
            let boxes: i32 = match form_boxes().parse() {
                Ok(b) => b,
                Err(_) => {
                    form_error.set("Invalid boxes count.".to_string());
                    return;
                }
            };
            let rate: f64 = match form_rate().parse() {
                Ok(r) => r,
                Err(_) => {
                    form_error.set("Invalid rate.".to_string());
                    return;
                }
            };
            let received: f64 = match form_received().parse() {
                Ok(r) => r,
                Err(_) => {
                    form_error.set("Invalid received amount.".to_string());
                    return;
                }
            };

            let total_eggs = boxes * 210;
            let total_amount = (total_eggs as f64) * rate;

            form_error.set(String::new());
            is_submitting.set(true);

            let new_sale = EggSale {
                id: form_id().unwrap_or_default(),
                date,
                party_name: party,
                party_id: form_party_id(),
                quantity_boxes: boxes,
                total_eggs,
                size: "Mixed".to_string(),
                gross_rate: rate,
                less_discount: 0.0,
                net_rate: rate,
                total_amount,
                received_amount: received,
                payment_mode: "Cash".to_string(),
                balance: total_amount - received,
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
            let trays: i32 = match form_boxes().parse() {
                Ok(b) => b,
                Err(_) => {
                    form_error.set("Invalid trays count.".to_string());
                    return;
                }
            };
            let rate: f64 = match form_rate().parse() {
                Ok(r) => r,
                Err(_) => {
                    form_error.set("Invalid rate.".to_string());
                    return;
                }
            };
            let received: f64 = match form_received().parse() {
                Ok(r) => r,
                Err(_) => {
                    form_error.set("Invalid received amount.".to_string());
                    return;
                }
            };

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
                return_trays: 0,
                empty_trays_balance: 0,
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
                    toast_api.success(tr("success"), ToastOptions::new().description("Sale record deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let q = search_query().trim().to_lowercase();

    let std_list = standard_sales.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let filtered_std: Vec<_> = std_list.into_iter().filter(|s| {
        let matches_month = if let Some(ref m) = selected_month() {
            s.date.starts_with(m)
        } else {
            true
        };
        let matches_search = if q.is_empty() {
            true
        } else {
            s.party_name.to_lowercase().contains(&q) || s.date.contains(&q)
        };
        matches_month && matches_search
    }).collect();

    let broken_list = broken_sales.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let filtered_broken: Vec<_> = broken_list.into_iter().filter(|s| {
        let matches_month = if let Some(ref m) = selected_month() {
            s.date.starts_with(m)
        } else {
            true
        };
        let matches_search = if q.is_empty() {
            true
        } else {
            s.bakery_name.to_lowercase().contains(&q) || s.date.contains(&q)
        };
        matches_month && matches_search
    }).collect();

    let title_str = tr("sales");
    let add_btn_str = tr("add-sale");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-3",
                div { class: "flex justify-between items-center",
                    div {
                        h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "💰 {title_str}" }
                        p { class: "text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Daily standard egg & broken egg sales dispatch and receivables" }
                    }
                    Button {
                        onclick: move |_| {
                            form_id.set(None);
                            form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                            form_party_name.set(String::new());
                            form_party_id.set(None);
                            form_boxes.set(String::new());
                            form_rate.set(String::new());
                            form_received.set(String::new());
                            form_error.set(String::new());
                            is_sheet_open.set(true);
                        },
                        "+ {add_btn_str}"
                    }
                }

                Input {
                    placeholder: "{search_ph}",
                    value: "{search_query}",
                    oninput: move |e: Event<FormData>| search_query.set(e.value())
                }

                div { class: "flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2",
                    MonthFilter {
                        selected: selected_month(),
                        onchange: move |m| selected_month.set(m)
                    }

                    // Tab Filter Pills (Standard Eggs vs Broken Eggs)
                    div { class: "flex p-1 bg-stone-100 dark:bg-stone-800/80 rounded-lg border border-stone-200/80 dark:border-stone-700/60 text-xs font-medium self-start sm:self-auto",
                        button {
                            class: if active_tab() == Some("standard".to_string()) {
                                "px-3 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all"
                            } else {
                                "px-3 py-1 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all"
                            },
                            onclick: move |_| active_tab.set(Some("standard".to_string())),
                            "Standard Eggs"
                        }
                        button {
                            class: if active_tab() == Some("broken".to_string()) {
                                "px-3 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all"
                            } else {
                                "px-3 py-1 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all"
                            },
                            onclick: move |_| active_tab.set(Some("broken".to_string())),
                            "Broken Eggs"
                        }
                    }
                }
            }

            // SECTION 2: SCROLLABLE CONTENT (Direct Card List matching Purchases)
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-20",

                    if active_tab() == Some("standard".to_string()) {
                        match standard_sales.cloned() {
                            Some(Ok(_)) if !filtered_std.is_empty() => rsx! {
                                div { class: "flex flex-col gap-3 mt-2",
                                    for sale in filtered_std {
                                        Card { key: "{sale.id}",
                                            CardHeader {
                                                div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                    span { "{sale.date}" }
                                                    span { "Size: {sale.size}" }
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
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Quantity" }
                                                            span { class: "font-semibold text-gray-800 dark:text-gray-200", "{sale.quantity_boxes} Boxes ({sale.total_eggs})" }
                                                        }
                                                        div { class: "flex flex-col",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Received" }
                                                            span { class: "font-semibold text-green-600 dark:text-green-500", "₹{sale.received_amount:.2}" }
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
                                                                    form_boxes.set(edit_sale.quantity_boxes.to_string());
                                                                    form_rate.set(edit_sale.gross_rate.to_string());
                                                                    form_received.set(edit_sale.received_amount.to_string());
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
                                div { class: "flex flex-col gap-3 mt-2",
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
                                                    div { class: "grid grid-cols-2 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                                        div { class: "flex flex-col",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Received" }
                                                            span { class: "font-semibold text-green-600 dark:text-green-500", "₹{sale.payment_received:.2}" }
                                                        }
                                                        div { class: "flex flex-col items-end",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Balance" }
                                                            span { class: "font-semibold text-red-500", "₹{sale.balance_amount:.2}" }
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
                                                                    form_boxes.set(edit_sale.trays_sold.to_string());
                                                                    form_rate.set(edit_sale.rate.to_string());
                                                                    form_received.set(edit_sale.payment_received.to_string());
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
                        description: "Are you sure you want to delete this sale record? The party ledger balance will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| {
                            if !is_deleting() {
                                delete_info.set(None);
                            }
                        }
                    }
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle {
                            if form_id().is_some() {
                                "Edit Sale"
                            } else if active_tab() == Some("standard".to_string()) {
                                "Add Standard Egg Sale"
                            } else {
                                "Add Broken Egg Sale"
                            }
                        }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "sale-date", "Date" }
                            Input {
                                r#type: "date",
                                value: "{form_date}",
                                oninput: move |e: Event<FormData>| form_date.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "sale-party", if active_tab() == Some("standard".to_string()) { "Select Customer Party" } else { "Select Bakery Party" } }
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
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "sale-boxes", if active_tab() == Some("standard".to_string()) { "Quantity (Boxes — 210 eggs/box)" } else { "Quantity (Trays — 30 eggs/tray)" } }
                            Input {
                                r#type: "number",
                                value: "{form_boxes}",
                                oninput: move |e: Event<FormData>| form_boxes.set(e.value()),
                                placeholder: if active_tab() == Some("standard".to_string()) { "e.g. 50 boxes" } else { "e.g. 20 trays" }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "sale-rate", if active_tab() == Some("standard".to_string()) { "Rate per Egg (₹/egg)" } else { "Rate per Tray (₹/tray)" } }
                            Input {
                                r#type: "number",
                                step: "0.01",
                                value: "{form_rate}",
                                oninput: move |e: Event<FormData>| form_rate.set(e.value()),
                                placeholder: if active_tab() == Some("standard".to_string()) { "e.g. 5.25" } else { "e.g. 130.00" }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "sale-received", "Received Payment Amount (₹)" }
                            Input {
                                r#type: "number",
                                step: "0.01",
                                value: "{form_received}",
                                oninput: move |e: Event<FormData>| form_received.set(e.value()),
                                placeholder: "0.00"
                            }
                        }

                        // Live Calculation Preview Box
                        {
                            let is_std = active_tab() == Some("standard".to_string());
                            let p_qty = form_boxes().parse::<f64>().unwrap_or(0.0);
                            let p_rate = form_rate().parse::<f64>().unwrap_or(0.0);
                            let p_rec = form_received().parse::<f64>().unwrap_or(0.0);
                            let tot_eggs = if is_std { (p_qty * 210.0) as i32 } else { (p_qty * 30.0) as i32 };
                            let tot_bill = if is_std { (tot_eggs as f64) * p_rate } else { p_qty * p_rate };
                            let vol_str = if is_std { format!("{} Boxes ({} Eggs)", p_qty as i32, tot_eggs) } else { format!("{} Trays ({} Eggs)", p_qty as i32, tot_eggs) };
                            let bal_due = tot_bill - p_rec;

                            rsx! {
                                div { class: "p-3 rounded-lg bg-blue-50/80 dark:bg-blue-950/40 border border-blue-200/80 dark:border-blue-800/60 text-xs space-y-1.5 mt-1",
                                    div { class: "font-medium text-blue-900 dark:text-blue-200 flex justify-between",
                                        span { if is_std { "📦 Total Egg Volume" } else { "🍳 Total Trays Volume" } }
                                        span { class: "font-bold", "{vol_str}" }
                                    }
                                    div { class: "font-medium text-blue-900 dark:text-blue-200 flex justify-between",
                                        span { "💵 Estimated Total Bill" }
                                        span { class: "font-bold text-sm text-blue-700 dark:text-blue-300", "₹ {tot_bill:.2}" }
                                    }
                                    div { class: "font-medium flex justify-between pt-1 border-t border-blue-200/60 dark:border-blue-800/40",
                                        span { class: "text-gray-600 dark:text-gray-300", "⏳ Balance Due After Payment" }
                                        span {
                                            class: if bal_due > 0.0 { "font-bold text-amber-600 dark:text-amber-400" } else { "font-bold text-emerald-600 dark:text-emerald-400" },
                                            "₹ {bal_due:.2}"
                                        }
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
