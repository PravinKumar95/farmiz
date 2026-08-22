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
use crate::models::MaterialPurchase;
use crate::services::*;

#[component]
pub fn Purchases() -> Element {
    let mut purchases = use_material_purchases();
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
    let mut form_material = use_signal(String::new);
    let mut form_party_name = use_signal(String::new);
    let mut form_party_id = use_signal(|| Option::<String>::None);
    let mut form_qty = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_advance = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

        let date = form_date().trim().to_string();
        let material = form_material().trim().to_string();
        let party = form_party_name().trim().to_string();

        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if material.is_empty() {
            form_error.set("Material name is required.".to_string());
            return;
        }
        if party.is_empty() {
            form_error.set("Supplier party is required.".to_string());
            return;
        }

        let qty: f64 = match form_qty().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid quantity.".to_string());
                return;
            }
        };
        let rate: f64 = match form_rate().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid rate.".to_string());
                return;
            }
        };
        let advance: f64 = match form_advance().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid advance amount.".to_string());
                return;
            }
        };

        let total_amount = qty * rate;
        let status = if advance >= total_amount { "PAID" } else { "PENDING" };

        form_error.set(String::new());
        is_submitting.set(true);

        let is_edit = form_id().is_some();
        let record = MaterialPurchase {
            id: form_id().unwrap_or_default(),
            date,
            material_name: material,
            party_name: party,
            party_id: form_party_id(),
            quantity_kg: qty,
            rate_per_kg: rate,
            total_amount,
            advance_paid: advance,
            status: status.to_string(),
            balance: total_amount - advance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/purchases/{}", id), &record).await
            } else {
                api.post("/api/purchases", &record).await
            };

            is_submitting.set(false);

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    purchases.restart();
                    let desc = if is_edit { "Purchase record updated successfully." } else { "Purchase record created successfully." };
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
                let res = api.delete(&format!("/api/purchases/{}", id)).await;
                if let Ok(_) = res {
                    delete_id.set(None);
                    purchases.restart();
                    toast_api.success(tr("success"), ToastOptions::new().description("Purchase record deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let records_list = purchases.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let q = search_query().trim().to_lowercase();
    let filtered_purchases: Vec<_> = records_list.into_iter().filter(|p| {
        let matches_date = date_range().matches(&p.date);
        let matches_search = if q.is_empty() {
            true
        } else {
            p.party_name.to_lowercase().contains(&q)
                || p.material_name.to_lowercase().contains(&q)
                || p.status.to_lowercase().contains(&q)
        };
        matches_date && matches_search
    }).collect();

    let total_qty_sum: f64 = filtered_purchases.iter().map(|p| p.quantity_kg).sum();
    let total_amount_sum: f64 = filtered_purchases.iter().map(|p| p.total_amount).sum();
    let total_advance_sum: f64 = filtered_purchases.iter().map(|p| p.advance_paid).sum();
    let total_balance_sum: f64 = filtered_purchases.iter().map(|p| p.balance).sum();

    let title_str = tr("purchases");
    let add_btn_str = tr("add-purchase");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex justify-between items-center gap-2",
                    div { class: "min-w-0",
                        h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "🛒 {title_str}" }
                        p { class: "hidden sm:block text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Track raw material purchases, supplier orders & payment balances" }
                    }
                    Button {
                        onclick: move |_| {
                            form_id.set(None);
                            form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                            form_material.set(String::new());
                            form_party_name.set(String::new());
                            form_party_id.set(None);
                            form_qty.set(String::new());
                            form_rate.set(String::new());
                            form_advance.set(String::new());
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

            // SECTION 2: SCROLLABLE CONTENT (Table View or Card View based on toggle)
            div { class: "flex-1 overflow-y-auto min-h-0 p-3 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-16 md:pb-20",
                    match purchases.cloned() {
                        Some(Ok(_)) if !filtered_purchases.is_empty() => rsx! {
                            if view_layout() == ViewLayout::Table {
                                // ── DATA TABLE VIEW ──
                                div { class: "mt-2",
                                    Card {
                                        div { class: "overflow-x-auto",
                                            table { class: "w-full text-sm text-left border-collapse",
                                                thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                    tr {
                                                        th { class: "px-4 py-3", "Date" }
                                                        th { class: "px-4 py-3", "Material" }
                                                        th { class: "px-4 py-3", "Supplier Party" }
                                                        th { class: "px-4 py-3 text-right", "Qty (kg)" }
                                                        th { class: "px-4 py-3 text-right", "Rate/kg" }
                                                        th { class: "px-4 py-3 text-right", "Total (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Advance (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Balance (₹)" }
                                                        th { class: "px-4 py-3 text-center", "Status" }
                                                        th { class: "px-4 py-3 text-right", "Actions" }
                                                    }
                                                }
                                                tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                    for item in filtered_purchases.iter() {
                                                        {
                                                            let edit_item = item.clone();
                                                            let del_id = item.id.clone();
                                                            rsx! {
                                                                tr {
                                                                    key: "{item.id}",
                                                                    class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                    td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{item.date}" }
                                                                    td { class: "px-4 py-3 font-semibold text-blue-600 dark:text-blue-400 whitespace-nowrap", "{item.material_name}" }
                                                                    td { class: "px-4 py-3 font-medium text-stone-800 dark:text-stone-200 whitespace-nowrap", "{item.party_name}" }
                                                                    td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300", "{item.quantity_kg:.1}" }
                                                                    td { class: "px-4 py-3 text-right font-medium text-stone-700 dark:text-stone-300", "₹{item.rate_per_kg:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-bold text-stone-900 dark:text-stone-100", "₹{item.total_amount:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400", "₹{item.advance_paid:.2}" }
                                                                    td { class: "px-4 py-3 text-right font-semibold text-rose-600 dark:text-rose-400", "₹{item.balance:.2}" }
                                                                    td { class: "px-4 py-3 text-center",
                                                                        span {
                                                                            class: if item.status == "PAID" {
                                                                                "px-2 py-0.5 rounded text-xs font-semibold bg-emerald-100 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800"
                                                                            } else {
                                                                                "px-2 py-0.5 rounded text-xs font-semibold bg-amber-100 dark:bg-amber-950 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-800"
                                                                            },
                                                                            "{item.status}"
                                                                        }
                                                                    }
                                                                    td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                                        div { class: "flex items-center justify-end gap-2",
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                onclick: move |_| {
                                                                                    form_id.set(Some(edit_item.id.clone()));
                                                                                    form_date.set(edit_item.date.clone());
                                                                                    form_material.set(edit_item.material_name.clone());
                                                                                    form_party_name.set(edit_item.party_name.clone());
                                                                                    form_party_id.set(edit_item.party_id.clone());
                                                                                    form_qty.set(edit_item.quantity_kg.to_string());
                                                                                    form_rate.set(edit_item.rate_per_kg.to_string());
                                                                                    form_advance.set(edit_item.advance_paid.to_string());
                                                                                    is_sheet_open.set(true);
                                                                                },
                                                                                "{edit_str}"
                                                                            }
                                                                            Button {
                                                                                variant: ButtonVariant::Destructive,
                                                                                onclick: move |_| delete_id.set(Some(del_id.clone())),
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
                                                        td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_purchases.len()} bills" }
                                                        td {}
                                                        td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{total_qty_sum:.1} kg" }
                                                        td {}
                                                        td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "₹{total_amount_sum:.2}" }
                                                        td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "₹{total_advance_sum:.2}" }
                                                        td { class: "px-4 py-3 text-right text-rose-700 dark:text-rose-300", "₹{total_balance_sum:.2}" }
                                                        td {}
                                                        td {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // ── CARD VIEW ──
                                div { class: "flex flex-col gap-3 mt-2",
                                    for item in filtered_purchases {
                                        Card { key: "{item.id}",
                                            CardHeader {
                                                div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                    span { "{item.date}" }
                                                    span { class: if item.status == "PAID" { "text-green-600 dark:text-green-400 font-semibold" } else { "text-amber-600 dark:text-amber-400 font-semibold" }, "{item.status}" }
                                                }
                                            }
                                            CardContent {
                                                div { class: "flex flex-col gap-3",
                                                    div { class: "flex justify-between items-baseline",
                                                        div { class: "flex flex-col",
                                                            span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{item.material_name}" }
                                                            span { class: "text-xs text-gray-500 dark:text-gray-400", "Supplier: {item.party_name}" }
                                                        }
                                                        span { class: "text-base font-bold text-blue-600 dark:text-blue-400", "₹ {item.total_amount:.2}" }
                                                    }
                                                    div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                                        div { class: "flex flex-col",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Quantity (kg)" }
                                                            span { class: "font-semibold text-gray-800 dark:text-gray-200", "{item.quantity_kg:.1} kg @ ₹{item.rate_per_kg:.2}" }
                                                        }
                                                        div { class: "flex flex-col",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Advance Paid" }
                                                            span { class: "font-semibold text-green-600 dark:text-green-500", "₹{item.advance_paid:.2}" }
                                                        }
                                                        div { class: "flex flex-col items-end",
                                                            span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Balance" }
                                                            span { class: "font-semibold text-red-500", "₹{item.balance:.2}" }
                                                        }
                                                    }
                                                }
                                            }
                                            CardFooter {
                                                div { class: "flex justify-end gap-2 w-full pt-1",
                                                    {
                                                        let edit_item = item.clone();
                                                        let del_id = item.id.clone();
                                                        rsx! {
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                onclick: move |_| {
                                                                    form_id.set(Some(edit_item.id.clone()));
                                                                    form_date.set(edit_item.date.clone());
                                                                    form_material.set(edit_item.material_name.clone());
                                                                    form_party_name.set(edit_item.party_name.clone());
                                                                    form_party_id.set(edit_item.party_id.clone());
                                                                    form_qty.set(edit_item.quantity_kg.to_string());
                                                                    form_rate.set(edit_item.rate_per_kg.to_string());
                                                                    form_advance.set(edit_item.advance_paid.to_string());
                                                                    is_sheet_open.set(true);
                                                                },
                                                                "{edit_str}"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: move |_| delete_id.set(Some(del_id.clone())),
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
                                icon: "🛒",
                                title: "No Material Purchases Found",
                                description: "No purchase records match your search criteria or date filter."
                            }
                        },
                        Some(Err(err)) => rsx! { ErrorState { message: err } },
                        None => rsx! { LoadingState {} }
                    }

                    ConfirmDialog {
                        is_open: delete_id().is_some(),
                        is_deleting: is_deleting(),
                        title: "Delete Purchase Record".to_string(),
                        description: "Are you sure you want to delete this purchase record? Supplier balance will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| {
                            if !is_deleting() {
                                delete_id.set(None);
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
                        SheetTitle { if form_id().is_some() { "Edit Purchase Record" } else { "Add Material Purchase" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "pur-date", "Date" }
                            Input {
                                r#type: "date",
                                value: "{form_date}",
                                oninput: move |e: Event<FormData>| form_date.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "pur-material", "Material Name" }
                            Input {
                                placeholder: "e.g. Maize, Soya DOC, Medicine",
                                value: "{form_material}",
                                oninput: move |e: Event<FormData>| form_material.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "pur-party", "Supplier Party" }
                            PartySelect {
                                value: form_party_id().unwrap_or_else(|| form_party_name()),
                                filter_type: Some("SUPPLIER".to_string()),
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
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "pur-qty", "Quantity (KG)" }
                                Input {
                                    r#type: "number",
                                    step: "0.1",
                                    value: "{form_qty}",
                                    oninput: move |e: Event<FormData>| form_qty.set(e.value()),
                                    placeholder: "0.0"
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "pur-rate", "Rate per KG" }
                                Input {
                                    r#type: "number",
                                    step: "0.01",
                                    value: "{form_rate}",
                                    oninput: move |e: Event<FormData>| form_rate.set(e.value()),
                                    placeholder: "0.00"
                                }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "pur-advance", "Advance Paid" }
                            Input {
                                r#type: "number",
                                step: "0.01",
                                value: "{form_advance}",
                                oninput: move |e: Event<FormData>| form_advance.set(e.value()),
                                placeholder: "0.00"
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
                            if form_id().is_some() { "Update Purchase" } else { "Save Purchase" }
                        }
                    }
                }
            }
        }
    }
}
