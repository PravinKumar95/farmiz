use dioxus::prelude::*;
use chrono::Utc;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::month_filter::MonthFilter;
use crate::components::party_select::PartySelect;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::MaterialPurchase;
use crate::services::*;

#[component]
pub fn Purchases() -> Element {
    let mut purchases = use_material_purchases();
    let api = use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let current_month_str = Utc::now().format("%Y-%m").to_string();
    let mut selected_month = use_signal(move || Some(current_month_str.clone()));
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

    let submit_handler = move |_| {
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

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    purchases.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    let confirm_delete = move |_| {
        if let Some(id) = delete_id() {
            spawn(async move {
                if let Ok(_) = api.delete(&format!("/api/purchases/{}", id)).await {
                    delete_id.set(None);
                    purchases.restart();
                }
            });
        }
    };

    let pur_list = purchases.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let q = search_query().trim().to_lowercase();
    let filtered_purchases: Vec<_> = pur_list.into_iter().filter(|p| {
        let matches_month = if let Some(ref m) = selected_month() {
            p.date.starts_with(m)
        } else {
            true
        };
        let matches_search = if q.is_empty() {
            true
        } else {
            p.material_name.to_lowercase().contains(&q)
                || p.party_name.to_lowercase().contains(&q)
                || p.date.contains(&q)
                || p.status.to_lowercase().contains(&q)
        };
        matches_month && matches_search
    }).collect();

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-3",
                div { class: "flex justify-between items-center",
                    h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Purchases" }
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
                        "Add Purchase"
                    }
                }

                Input {
                    placeholder: "🔍 Search by supplier, material, status...",
                    value: "{search_query}",
                    oninput: move |e: Event<FormData>| search_query.set(e.value())
                }
                MonthFilter {
                    selected: selected_month(),
                    onchange: move |m| selected_month.set(m)
                }
            }

            // SECTION 2: SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
                    match purchases.cloned() {
                        Some(Ok(_)) if !filtered_purchases.is_empty() => rsx! {
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
                                                            "Edit"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                delete_id.set(Some(del_id.clone()));
                                                            },
                                                            "Delete"
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
                        title: "Delete Purchase Record".to_string(),
                        description: "Are you sure you want to delete this purchase record? Supplier balance will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| delete_id.set(None)
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
                            onclick: move |_| is_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            onclick: submit_handler,
                            if form_id().is_some() { "Update Purchase" } else { "Save Purchase" }
                        }
                    }
                }
            }
        }
    }
}
