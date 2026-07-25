use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::*;
use crate::models::MaterialPurchase;
use crate::components::empty_state::{EmptyState, LoadingState, ErrorState};

#[component]
pub fn Purchases() -> Element {
    let parties = use_parties();
    let mut purchases = use_material_purchases();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut form_id = use_signal(|| Option::<String>::None);

    let mut form_date = use_signal(String::new);
    let mut form_material = use_signal(String::new);
    let mut form_party = use_signal(String::new);
    let mut form_qty = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_advance = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let material = form_material().trim().to_string();
        let party = form_party().trim().to_string();
        
        if date.is_empty() { form_error.set("Date required".to_string()); return; }
        if material.is_empty() { form_error.set("Material required".to_string()); return; }
        if party.is_empty() { form_error.set("Party required".to_string()); return; }
        
        let qty: f64 = match form_qty().parse() {
            Ok(v) => v,
            Err(_) => { form_error.set("Invalid quantity".to_string()); return; }
        };
        let rate: f64 = match form_rate().parse() {
            Ok(v) => v,
            Err(_) => { form_error.set("Invalid rate".to_string()); return; }
        };
        let advance: f64 = match form_advance().parse() {
            Ok(v) => v,
            Err(_) => { form_error.set("Invalid advance amount".to_string()); return; }
        };
        
        let total_amount = qty * rate;
        let status = if advance >= total_amount { "PAID" } else { "PENDING" };
        
        let new_record = MaterialPurchase {
            id: form_id().unwrap_or_default(),
            date,
            material_name: material,
            party_name: party,
            quantity_kg: qty,
            rate_per_kg: rate,
            total_amount,
            advance_paid: advance,
            status: status.to_string(),
            balance: total_amount - advance,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/purchases/{}", id), &new_record).await
            } else {
                api.post("/api/purchases", &new_record).await
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

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            datalist { id: "parties-list",
                for party in parties.cloned().and_then(|r| r.ok()).unwrap_or_default() {
                    option { value: "{party.name}" }
                }
            }
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Material Purchases" }
                Button { 
                    onclick: move |_| {
                        form_id.set(None);
                        form_date.set(String::new());
                        form_material.set(String::new());
                        form_party.set(String::new());
                        form_qty.set(String::new());
                        form_rate.set(String::new());
                        form_advance.set(String::new());
                        is_sheet_open.set(true);
                    },
                    "Add Purchase" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader { SheetTitle { if form_id().is_some() { "Edit Material Purchase" } else { "Add Material Purchase" } } }
                    div { class: "flex flex-col gap-4 px-6 py-4 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "date", "Date" }
                            Input { value: "{form_date}", oninput: move |e: FormEvent| form_date.set(e.value()), placeholder: "YYYY-MM-DD" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "material", "Material Name" }
                            Input { value: "{form_material}", oninput: move |e: FormEvent| form_material.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party", "Party Name" }
                            Input { list: "parties-list", value: "{form_party}", oninput: move |e: FormEvent| form_party.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "qty", "Quantity (KG)" }
                            Input { value: "{form_qty}", oninput: move |e: FormEvent| form_qty.set(e.value()), placeholder: "0.00" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "rate", "Rate per KG" }
                            Input { value: "{form_rate}", oninput: move |e: FormEvent| form_rate.set(e.value()), placeholder: "0.00" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "advance", "Advance Paid" }
                            Input { value: "{form_advance}", oninput: move |e: FormEvent| form_advance.set(e.value()), placeholder: "0.00" }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, if form_id().is_some() { "Update Purchase" } else { "Save Purchase" } }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            match purchases.cloned() {
                Some(Ok(list)) if !list.is_empty() => rsx! {
                    div { class: "flex flex-col gap-3 mt-4",
                        for purchase in list {
                            Card { key: "{purchase.id}",
                                CardHeader {
                                    div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                        span { "{purchase.date}" }
                                        div {
                                            class: if purchase.status == "PAID" {
                                                "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 px-2 py-0.5 rounded-full text-xs font-semibold"
                                            } else {
                                                "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 px-2 py-0.5 rounded-full text-xs font-semibold"
                                            },
                                            "{purchase.status}"
                                        }
                                    }
                                }
                                CardContent {
                                    div { class: "flex flex-col gap-3",
                                        div { class: "flex justify-between items-baseline",
                                            span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{purchase.material_name}" }
                                            span { class: "text-sm text-gray-500 dark:text-gray-400 font-medium", "{purchase.party_name}" }
                                        }
                                        div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                            div { class: "flex flex-col",
                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Quantity" }
                                                span { class: "font-semibold text-gray-800 dark:text-gray-200", "{purchase.quantity_kg} KG @ ₹{purchase.rate_per_kg:.2}" }
                                            }
                                            div { class: "flex flex-col",
                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Advance" }
                                                span { class: "font-semibold text-green-600 dark:text-green-500", "₹{purchase.advance_paid:.2}" }
                                            }
                                            div { class: "flex flex-col items-end",
                                                span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Balance" }
                                                span { class: "font-semibold text-red-500", "₹{purchase.balance:.2}" }
                                            }
                                        }
                                    }
                                }
                                CardFooter {
                                    div { class: "flex justify-end gap-2 w-full pt-1",
                                        {
                                            let edit_purchase = purchase.clone();
                                            let delete_id = purchase.id.clone();
                                            rsx! {
                                                Button {
                                                    variant: ButtonVariant::Outline,
                                                    onclick: move |_| {
                                                        form_id.set(Some(edit_purchase.id.clone()));
                                                        form_date.set(edit_purchase.date.clone());
                                                        form_material.set(edit_purchase.material_name.clone());
                                                        form_party.set(edit_purchase.party_name.clone());
                                                        form_qty.set(edit_purchase.quantity_kg.to_string());
                                                        form_rate.set(edit_purchase.rate_per_kg.to_string());
                                                        form_advance.set(edit_purchase.advance_paid.to_string());
                                                        is_sheet_open.set(true);
                                                    },
                                                    "Edit"
                                                }
                                                Button {
                                                    variant: ButtonVariant::Outline,
                                                    onclick: move |_| {
                                                        let id = delete_id.clone();
                                                        spawn(async move {
                                                            let _ = api.delete(&format!("/api/purchases/{}", id)).await;
                                                            purchases.restart();
                                                        });
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
                        title: "No Material Purchases",
                        description: "No purchase records found. Click 'Add Material Purchase' above to add a new purchase."
                    }
                },
                Some(Err(err)) => rsx! {
                    ErrorState {
                        message: err,
                        on_retry: move |_| { purchases.restart(); }
                    }
                },
                None => rsx! {
                    LoadingState {}
                }
            }
        }
    }
}
