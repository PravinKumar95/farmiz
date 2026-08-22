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
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::FeedBatch;
use crate::services::*;

#[component]
pub fn Feed() -> Element {
    let mut batches = use_feed_batches();
    let api = use_auth();
    let toast_api = use_toast();
    let mut is_sheet_open = use_signal(|| false);

    let mut view_layout = use_signal(ViewLayout::default);
    let mut date_range = use_signal(DateRange::default);
    let mut search_query = use_signal(String::new);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_batch_id = use_signal(String::new);
    let mut form_type = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_total = use_signal(String::new);
    let mut form_payment = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

        let date = form_date().trim().to_string();
        let batch_id = form_batch_id().trim().to_string();
        let feed_type = form_type().trim().to_string();

        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if batch_id.is_empty() {
            form_error.set("Batch ID / Count is required.".to_string());
            return;
        }

        let rate: f64 = match form_rate().parse() {
            Ok(r) => r,
            Err(_) => {
                form_error.set("Invalid rate.".to_string());
                return;
            }
        };
        let total: f64 = match form_total().parse() {
            Ok(t) => t,
            Err(_) => {
                form_error.set("Invalid total amount.".to_string());
                return;
            }
        };
        let payment: f64 = match form_payment().parse() {
            Ok(p) => p,
            Err(_) => 0.0,
        };

        form_error.set(String::new());
        is_submitting.set(true);

        let is_edit = form_id().is_some();
        let new_batch = FeedBatch {
            id: form_id().unwrap_or_default(),
            date,
            batch_id,
            feed_type,
            rate,
            total_amount: total,
            payment,
            opening_balance: 0.0,
            closing_balance: total - payment,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/feed/{}", id), &new_batch).await
            } else {
                api.post("/api/feed", &new_batch).await
            };

            is_submitting.set(false);

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    batches.restart();
                    let desc = if is_edit { "Feed batch updated successfully." } else { "Feed batch created successfully." };
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
                let res = api.delete(&format!("/api/feed/{}", id)).await;
                if let Ok(_) = res {
                    delete_id.set(None);
                    batches.restart();
                    toast_api.success(tr("success"), ToastOptions::new().description("Feed batch deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let batch_list = batches.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let q = search_query().trim().to_lowercase();
    let filtered_batches: Vec<_> = batch_list.into_iter().filter(|b| {
        let matches_date = date_range().matches(&b.date);
        let matches_search = if q.is_empty() {
            true
        } else {
            b.feed_type.to_lowercase().contains(&q)
                || b.batch_id.to_lowercase().contains(&q)
                || b.date.contains(&q)
        };
        matches_date && matches_search
    }).collect();

    let total_feed_amount: f64 = filtered_batches.iter().map(|b| b.total_amount).sum();
    let total_feed_payment: f64 = filtered_batches.iter().map(|b| b.payment).sum();
    let total_feed_balance: f64 = filtered_batches.iter().map(|b| b.closing_balance).sum();

    let title_str = tr("feed-mill");
    let add_btn_str = tr("add-feed-batch");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex justify-between items-center gap-2",
                    div { class: "min-w-0",
                        h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "🌾 {title_str}" }
                        p { class: "hidden sm:block text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Track feed production batches, rate calculations & mill payments" }
                    }
                    Button {
                        onclick: move |_| {
                            form_id.set(None);
                            form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                            form_batch_id.set(String::new());
                            form_type.set("LAYER".to_string());
                            form_rate.set("110".to_string());
                            form_total.set(String::new());
                            form_payment.set("0.0".to_string());
                            form_error.set(String::new());
                            is_sheet_open.set(true);
                        },
                        "+ {add_btn_str}"
                    }
                }

                div { class: "flex flex-col gap-2",
                    div { class: "flex items-center gap-2",
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
                    match batches.cloned() {
                        Some(Ok(_)) if !filtered_batches.is_empty() => rsx! {
                            if view_layout() == ViewLayout::Table {
                                // ── FEED BATCHES DATA TABLE VIEW ──
                                div { class: "mt-2",
                                    Card {
                                        div { class: "overflow-x-auto",
                                            table { class: "w-full text-sm text-left border-collapse",
                                                thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                    tr {
                                                        th { class: "px-4 py-3", "Date" }
                                                        th { class: "px-4 py-3", "Batch ID(s)" }
                                                        th { class: "px-4 py-3", "Feed Type" }
                                                        th { class: "px-4 py-3 text-right", "Rate/Batch" }
                                                        th { class: "px-4 py-3 text-right", "Total (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Payment (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Closing Balance (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Actions" }
                                                    }
                                                }
                                                tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                    for item in filtered_batches.iter() {
                                                        {
                                                            let edit_item = item.clone();
                                                            let del_id = item.id.clone();
                                                            rsx! {
                                                                tr {
                                                                    key: "{item.id}",
                                                                    class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                    td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{item.date}" }
                                                                    td { class: "px-4 py-3 font-semibold text-amber-700 dark:text-amber-400 whitespace-nowrap",
                                                                        span { class: "px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/80 border border-amber-200 dark:border-amber-800 font-mono text-xs", "{item.batch_id}" }
                                                                    }
                                                                    td { class: "px-4 py-3 font-semibold text-stone-800 dark:text-stone-200 whitespace-nowrap", "{item.feed_type}" }
                                                                    td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300", "₹{item.rate:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-bold text-stone-900 dark:text-stone-100", "₹{item.total_amount:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400", "₹{item.payment:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-semibold text-rose-600 dark:text-rose-400", "₹{item.closing_balance:.2}" }
                                                                    td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                                        div { class: "flex items-center justify-end gap-2",
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                onclick: move |_| {
                                                                                    form_id.set(Some(edit_item.id.clone()));
                                                                                    form_date.set(edit_item.date.clone());
                                                                                    form_batch_id.set(edit_item.batch_id.clone());
                                                                                    form_type.set(edit_item.feed_type.clone());
                                                                                    form_rate.set(edit_item.rate.to_string());
                                                                                    form_total.set(edit_item.total_amount.to_string());
                                                                                    form_payment.set(edit_item.payment.to_string());
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
                                                tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                                    tr {
                                                        td { class: "px-4 py-3", "TOTAL" }
                                                        td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_batches.len()} records" }
                                                        td {}
                                                        td {}
                                                        td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "₹{total_feed_amount:.2}" }
                                                        td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "₹{total_feed_payment:.2}" }
                                                        td { class: "px-4 py-3 text-right text-rose-700 dark:text-rose-300", "₹{total_feed_balance:.2}" }
                                                        td {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // ── FEED BATCHES CARD VIEW ──
                                div { class: "flex flex-col gap-3 mt-2",
                                    for item in filtered_batches {
                                        Card { key: "{item.id}",
                                            CardHeader {
                                                div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                    span { "🗓️ {item.date}" }
                                                    span { class: "px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/80 text-amber-700 dark:text-amber-300 font-semibold border border-amber-200 dark:border-amber-800", "Batches: {item.batch_id}" }
                                                }
                                            }
                                            CardContent {
                                                div { class: "flex flex-col gap-3",
                                                    div { class: "flex justify-between items-baseline",
                                                        span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{item.feed_type}" }
                                                        span { class: "text-base font-bold text-blue-600 dark:text-blue-400", "Total: ₹{item.total_amount:.2}" }
                                                    }
                                                    div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                                        div { class: "flex flex-col",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Rate per Batch" }
                                                            span { class: "font-semibold text-stone-900 dark:text-stone-100", "₹{item.rate:.2}" }
                                                        }
                                                        div { class: "flex flex-col items-center",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Payment Made" }
                                                            span { class: "font-semibold text-emerald-600 dark:text-emerald-400", "₹{item.payment:.2}" }
                                                        }
                                                        div { class: "flex flex-col items-end",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Closing Balance" }
                                                            span { class: "font-semibold text-rose-600 dark:text-rose-400", "₹{item.closing_balance:.2}" }
                                                        }
                                                    }
                                                }
                                            }
                                            CardFooter {
                                                div { class: "flex justify-end gap-2 w-full pt-1 border-t border-stone-100 dark:border-stone-800/60 mt-2",
                                                    {
                                                        let edit_item = item.clone();
                                                        let del_id = item.id.clone();
                                                        rsx! {
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                onclick: move |_| {
                                                                    form_id.set(Some(edit_item.id.clone()));
                                                                    form_date.set(edit_item.date.clone());
                                                                    form_batch_id.set(edit_item.batch_id.clone());
                                                                    form_type.set(edit_item.feed_type.clone());
                                                                    form_rate.set(edit_item.rate.to_string());
                                                                    form_total.set(edit_item.total_amount.to_string());
                                                                    form_payment.set(edit_item.payment.to_string());
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
                            }
                        },
                        Some(Ok(_)) => rsx! {
                            EmptyState {
                                icon: "🌾",
                                title: "No Feed Batches Found",
                                description: "No feed production batches match your search criteria or date filter."
                            }
                        },
                        Some(Err(err)) => rsx! { ErrorState { message: err } },
                        None => rsx! { LoadingState {} }
                    }
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if form_id().is_some() { "Edit Feed Batch" } else { "Add Feed Batch" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "feed-date", "Date" }
                            Input {
                                r#type: "date",
                                value: "{form_date}",
                                oninput: move |e: Event<FormData>| form_date.set(e.value())
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "feed-batch", "Batches / Batch Count" }
                                Input {
                                    r#type: "number",
                                    step: "0.5",
                                    placeholder: "e.g. 7 or 13.5",
                                    value: "{form_batch_id}",
                                    oninput: move |e: Event<FormData>| {
                                        let val = e.value();
                                        form_batch_id.set(val.clone());
                                        if let (Ok(b), Ok(r)) = (val.parse::<f64>(), form_rate().parse::<f64>()) {
                                            form_total.set(format!("{:.2}", b * r));
                                        }
                                    }
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "feed-type", "Feed Type" }
                                Input {
                                    placeholder: "e.g. LAYER, CHICK",
                                    value: "{form_type}",
                                    oninput: move |e: Event<FormData>| form_type.set(e.value())
                                }
                            }
                        }
                        div { class: "grid grid-cols-3 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "feed-rate", "Rate (₹)" }
                                Input {
                                    r#type: "number",
                                    placeholder: "e.g. 110 or 310",
                                    value: "{form_rate}",
                                    oninput: move |e: Event<FormData>| {
                                        let val = e.value();
                                        form_rate.set(val.clone());
                                        if let (Ok(b), Ok(r)) = (form_batch_id().parse::<f64>(), val.parse::<f64>()) {
                                            form_total.set(format!("{:.2}", b * r));
                                        }
                                    }
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "feed-total", "Total Amount (₹)" }
                                Input {
                                    r#type: "number",
                                    placeholder: "Auto-calculated",
                                    value: "{form_total}",
                                    oninput: move |e: Event<FormData>| form_total.set(e.value())
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "feed-payment", "Payment Paid (₹)" }
                                Input {
                                    r#type: "number",
                                    placeholder: "e.g. 700",
                                    value: "{form_payment}",
                                    oninput: move |e: Event<FormData>| form_payment.set(e.value())
                                }
                            }
                        }

                        // Live Milling Cost Calculation Preview
                        {
                            let p_batches = form_batch_id().parse::<f64>().unwrap_or(0.0);
                            let p_rate = form_rate().parse::<f64>().unwrap_or(0.0);
                            let p_pay = form_payment().parse::<f64>().unwrap_or(0.0);
                            let tot = if let Ok(t) = form_total().parse::<f64>() { t } else { p_batches * p_rate };
                            let bal = tot - p_pay;
                            let batch_vol_str = format!("{:.1} Batches", p_batches);
                            let tot_str = format!("₹ {:.2}", tot);
                            let bal_str = format!("₹ {:.2}", bal);

                            rsx! {
                                div { class: "p-3 rounded-lg bg-amber-50/80 dark:bg-amber-950/40 border border-amber-200/80 dark:border-amber-800/60 text-xs space-y-1.5 mt-1",
                                    div { class: "font-medium text-amber-900 dark:text-amber-200 flex justify-between",
                                        span { "🌾 Production Volume" }
                                        span { class: "font-bold", "{batch_vol_str}" }
                                    }
                                    div { class: "font-medium text-amber-900 dark:text-amber-200 flex justify-between",
                                        span { "💵 Total Milling Cost" }
                                        span { class: "font-bold text-sm text-amber-700 dark:text-amber-300", "{tot_str}" }
                                    }
                                    div { class: "font-medium flex justify-between pt-1 border-t border-amber-200/60 dark:border-amber-800/40",
                                        span { class: "text-gray-600 dark:text-gray-300", "⏳ Pending Milling Balance" }
                                        span {
                                            class: if bal > 0.0 { "font-bold text-rose-600 dark:text-rose-400" } else { "font-bold text-emerald-600 dark:text-emerald-400" },
                                            "{bal_str}"
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
                            if form_id().is_some() { "Update Batch" } else { "Save Batch" }
                        }
                    }
                }
            }

            ConfirmDialog {
                is_open: delete_id().is_some(),
                is_deleting: is_deleting(),
                title: "Delete Feed Batch".to_string(),
                description: "Are you sure you want to delete this feed batch record?".to_string(),
                onconfirm: confirm_delete,
                oncancel: move |_| {
                    if !is_deleting() {
                        delete_id.set(None);
                    }
                }
            }
        }
    }
}
