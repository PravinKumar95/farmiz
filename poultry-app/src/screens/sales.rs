use dioxus::prelude::*;
use dioxus_sdk::storage::{use_storage, LocalStorage};

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::{use_broken_egg_sales, use_egg_sales, create_egg_sale, create_broken_egg_sale};
use crate::models::{EggSale, BrokenEggSale};

#[component]
pub fn Sales() -> Element {
    let mut active_tab = use_signal(|| Some("standard".to_string()));
    let mut standard_sales = use_egg_sales();
    let mut broken_sales = use_broken_egg_sales();
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), String::new);
    let mut is_sheet_open = use_signal(|| false);

    // Form state shared
    let mut form_date = use_signal(|| String::new());
    let mut form_party = use_signal(|| String::new());
    let mut form_boxes = use_signal(|| String::new());
    let mut form_rate = use_signal(|| String::new());
    let mut form_received = use_signal(|| String::new());
    let mut form_error = use_signal(|| String::new());

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
        
        let token = auth_token.read().clone();
        
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
                id: String::new(),
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
                match create_egg_sale(&token, &new_sale).await {
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
                id: String::new(),
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
                match create_broken_egg_sale(&token, &new_sale).await {
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
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Sales" }
                Button { 
                    onclick: move |_| is_sheet_open.set(true),
                    "Add Sale" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if active_tab() == Some("standard".to_string()) { "Add Standard Egg Sale" } else { "Add Broken Egg Sale" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "date", "Date" }
                            Input { value: "{form_date}", oninput: move |e: FormEvent| form_date.set(e.value()), placeholder: "YYYY-MM-DD" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party", if active_tab() == Some("standard".to_string()) { "Party Name" } else { "Bakery Name" } }
                            Input { value: "{form_party}", oninput: move |e: FormEvent| form_party.set(e.value()) }
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
                        Button { onclick: submit_handler, "Save Sale" }
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
                    div { class: "flex flex-col gap-3 mt-4",
                        for sale in standard_sales.cloned().unwrap_or_default() {
                            Card { key: "{sale.id}",
                                CardHeader {
                                    div { class: "flex justify-between items-center text-sm",
                                        span { class: "text-muted-foreground", "{sale.date}" }
                                        span { class: "font-semibold text-primary", "₹ {sale.total_amount:.2}" }
                                    }
                                }
                                CardContent {
                                    div { class: "flex flex-col gap-1",
                                        div { class: "font-bold text-lg", "{sale.party_name}" }
                                        div { class: "text-sm text-muted-foreground flex justify-between",
                                            span { "{sale.quantity_boxes} Boxes ({sale.total_eggs} eggs)" }
                                            span { "Size: {sale.size}" }
                                        }
                                    }
                                }
                                CardFooter { class: "bg-muted/50 pt-4 rounded-b-xl flex justify-between text-sm",
                                    div { class: "flex flex-col",
                                        span { class: "text-muted-foreground", "Received" }
                                        span { class: "font-medium text-green-600", "₹ {sale.received_amount:.2}" }
                                    }
                                    div { class: "flex flex-col items-end",
                                        span { class: "text-muted-foreground", "Balance" }
                                        span { class: "font-medium text-red-500", "₹ {sale.balance:.2}" }
                                    }
                                }
                            }
                        }
                    }
                }

                TabContent { value: "broken", index: 1usize,
                    div { class: "flex flex-col gap-3 mt-4",
                        for sale in broken_sales.cloned().unwrap_or_default() {
                            Card { key: "{sale.id}",
                                CardHeader {
                                    div { class: "flex justify-between items-center text-sm",
                                        span { class: "text-muted-foreground", "{sale.date}" }
                                        span { class: "font-semibold text-primary", "₹ {sale.amount:.2}" }
                                    }
                                }
                                CardContent {
                                    div { class: "flex flex-col gap-1",
                                        div { class: "font-bold text-lg", "{sale.bakery_name}" }
                                        div { class: "text-sm text-muted-foreground",
                                            "{sale.trays_sold} Trays @ ₹{sale.rate:.2}/tray"
                                        }
                                    }
                                }
                                CardFooter { class: "bg-muted/50 pt-4 rounded-b-xl flex justify-between text-sm",
                                    div { class: "flex flex-col",
                                        span { class: "text-muted-foreground", "Received" }
                                        span { class: "font-medium text-green-600", "₹ {sale.payment_received:.2}" }
                                    }
                                    div { class: "flex flex-col items-end",
                                        span { class: "text-muted-foreground", "Balance" }
                                        span { class: "font-medium text-red-500", "₹ {sale.balance_amount:.2}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
