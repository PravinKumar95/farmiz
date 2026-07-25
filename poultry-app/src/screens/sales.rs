use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::*;
use crate::models::{EggSale, BrokenEggSale};
use crate::components::button::ButtonVariant;
use crate::components::empty_state::{EmptyState, LoadingState, ErrorState};

#[component]
pub fn Sales() -> Element {
    let parties = use_parties();
    let mut active_tab = use_signal(|| Some("standard".to_string()));
    let mut standard_sales = use_egg_sales();
    let mut broken_sales = use_broken_egg_sales();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut form_id = use_signal(|| Option::<String>::None);
    let mut form_date = use_signal(String::new);
    let mut form_party = use_signal(String::new);
    let mut form_boxes = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_received = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let party = form_party().trim().to_string();
        
        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if party.len() < 2 {
            form_error.set("Party name is required.".to_string());
            return;
        }
        
        if active_tab() == Some("standard".to_string()) {
            let boxes: i32 = match form_boxes().parse() {
                Ok(b) => b,
                Err(_) => { form_error.set("Invalid boxes count.".to_string()); return; }
            };
            let rate: f64 = match form_rate().parse() {
                Ok(r) => r,
                Err(_) => { form_error.set("Invalid rate.".to_string()); return; }
            };
            let received: f64 = match form_received().parse() {
                Ok(r) => r,
                Err(_) => { form_error.set("Invalid received amount.".to_string()); return; }
            };
            
            let total_eggs = boxes * 210;
            let total_amount = (total_eggs as f64) * rate; // Simplified logic
            
            let new_sale = EggSale {
                id: form_id().unwrap_or_default(),
                date,
                party_name: party,
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
                created_at: None,
            };

            spawn(async move {
                let res = if let Some(id) = form_id() {
                    api.put(&format!("/api/sales/egg/{}", id), &new_sale).await
                } else {
                    api.post("/api/sales/egg", &new_sale).await
                };
                match res {
                    Ok(_) => {
                        is_sheet_open.set(false);
                        form_error.set(String::new());
                        standard_sales.restart();
                    }
                    Err(e) => form_error.set(e),
                }
            });
        } else {
            let trays: i32 = match form_boxes().parse() {
                Ok(b) => b,
                Err(_) => { form_error.set("Invalid trays count.".to_string()); return; }
            };
            let rate: f64 = match form_rate().parse() {
                Ok(r) => r,
                Err(_) => { form_error.set("Invalid rate.".to_string()); return; }
            };
            let received: f64 = match form_received().parse() {
                Ok(r) => r,
                Err(_) => { form_error.set("Invalid received amount.".to_string()); return; }
            };
            
            let total_amount = (trays as f64) * rate;
            
            let new_sale = BrokenEggSale {
                id: form_id().unwrap_or_default(),
                date,
                bakery_name: party,
                trays_sold: trays,
                rate,
                amount: total_amount,
                payment_received: received,
                return_trays: 0,
                empty_trays_balance: 0,
                balance_amount: total_amount - received,
                created_at: None,
            };

            spawn(async move {
                let res = if let Some(id) = form_id() {
                    api.put(&format!("/api/sales/broken/{}", id), &new_sale).await
                } else {
                    api.post("/api/sales/broken", &new_sale).await
                };
                match res {
                    Ok(_) => {
                        is_sheet_open.set(false);
                        form_error.set(String::new());
                        broken_sales.restart();
                    }
                    Err(e) => form_error.set(e),
                }
            });
        }
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            datalist { id: "parties-list",
                for party in parties.cloned().and_then(|r| r.ok()).unwrap_or_default() {
                    option { value: "{party.name}" }
                }
            }
            
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Sales" }
                Button { 
                    onclick: move |_| {
                        form_id.set(None);
                        form_date.set(String::new());
                        form_party.set(String::new());
                        form_boxes.set(String::new());
                        form_rate.set(String::new());
                        form_received.set(String::new());
                        is_sheet_open.set(true);
                    },
                    "Add Sale" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if form_id().is_some() { "Edit Sale" } else if active_tab() == Some("standard".to_string()) { "Add Standard Egg Sale" } else { "Add Broken Egg Sale" } }
                    }
                    div { class: "flex flex-col gap-4 px-6 py-4 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "date", "Date" }
                            Input { value: "{form_date}", oninput: move |e: FormEvent| form_date.set(e.value()), placeholder: "YYYY-MM-DD" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party", if active_tab() == Some("standard".to_string()) { "Party Name" } else { "Bakery Name" } }
                            Input { list: "parties-list", value: "{form_party}", oninput: move |e: FormEvent| form_party.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "qty", if active_tab() == Some("standard".to_string()) { "Boxes" } else { "Trays" } }
                            Input { value: "{form_boxes}", oninput: move |e: FormEvent| form_boxes.set(e.value()), placeholder: "0" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "rate", "Rate" }
                            Input { value: "{form_rate}", oninput: move |e: FormEvent| form_rate.set(e.value()), placeholder: "0.00" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "received", "Received Amount" }
                            Input { value: "{form_received}", oninput: move |e: FormEvent| form_received.set(e.value()), placeholder: "0.00" }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, if form_id().is_some() { "Update Sale" } else { "Save Sale" } }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            Tabs {
                value: active_tab,
                on_value_change: move |v: String| active_tab.set(Some(v)),
                TabList { class: "grid w-full grid-cols-2",
                    TabTrigger { value: "standard", index: 0usize, "Standard Eggs" }
                    TabTrigger { value: "broken", index: 1usize, "Broken Eggs" }
                }

                TabContent { value: "standard", index: 0usize,
                    match standard_sales.cloned() {
                        Some(Ok(list)) if !list.is_empty() => rsx! {
                            div { class: "flex flex-col gap-3 mt-4",
                                for sale in list {
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
                                                                form_party.set(edit_sale.party_name.clone());
                                                                form_boxes.set(edit_sale.quantity_boxes.to_string());
                                                                form_rate.set(edit_sale.gross_rate.to_string());
                                                                form_received.set(edit_sale.received_amount.to_string());
                                                                is_sheet_open.set(true);
                                                            },
                                                            "Edit"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                let id = delete_id.clone();
                                                                spawn(async move {
                                                                    let _ = api.delete(&format!("/api/sales/egg/{}", id)).await;
                                                                    standard_sales.restart();
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
                                icon: "🥚",
                                title: "No Standard Egg Sales",
                                description: "No standard egg sales records found. Click 'Add Sale' above to record a sale."
                            }
                        },
                        Some(Err(err)) => rsx! {
                            ErrorState {
                                message: err,
                                on_retry: move |_| { standard_sales.restart(); }
                            }
                        },
                        None => rsx! {
                            LoadingState {}
                        }
                    }
                }

                TabContent { value: "broken", index: 1usize,
                    match broken_sales.cloned() {
                        Some(Ok(list)) if !list.is_empty() => rsx! {
                            div { class: "flex flex-col gap-3 mt-4",
                                for sale in list {
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
                                                                form_party.set(edit_sale.bakery_name.clone());
                                                                form_boxes.set(edit_sale.trays_sold.to_string());
                                                                form_rate.set(edit_sale.rate.to_string());
                                                                form_received.set(edit_sale.payment_received.to_string());
                                                                is_sheet_open.set(true);
                                                            },
                                                            "Edit"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                let id = delete_id.clone();
                                                                spawn(async move {
                                                                    let _ = api.delete(&format!("/api/sales/broken/{}", id)).await;
                                                                    broken_sales.restart();
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
                                icon: "🍳",
                                title: "No Broken Egg Sales",
                                description: "No broken egg sales records found. Click 'Add Sale' above to record a sale."
                            }
                        },
                        Some(Err(err)) => rsx! {
                            ErrorState {
                                message: err,
                                on_retry: move |_| { broken_sales.restart(); }
                            }
                        },
                        None => rsx! {
                            LoadingState {}
                        }
                    }
                }
            }
        }
    }
}
