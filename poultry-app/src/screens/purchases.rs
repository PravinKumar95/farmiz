use dioxus::prelude::*;
use dioxus_sdk::storage::{use_storage, LocalStorage};

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::{use_material_purchases, create_material_purchase};
use crate::models::MaterialPurchase;

#[component]
pub fn Purchases() -> Element {
    let mut purchases = use_material_purchases();
    let auth_token = use_storage::<LocalStorage, _>("auth_token".to_string(), || None::<String>);
    let mut is_sheet_open = use_signal(|| false);

    let mut form_date = use_signal(|| String::new());
    let mut form_material = use_signal(|| String::new());
    let mut form_party = use_signal(|| String::new());
    let mut form_qty = use_signal(|| String::new());
    let mut form_rate = use_signal(|| String::new());
    let mut form_advance = use_signal(|| String::new());
    let mut form_error = use_signal(|| String::new());

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
            id: String::new(),
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

        let token = auth_token.read().clone().unwrap_or_default();
        spawn(async move {
            match create_material_purchase(&token, &new_record).await {
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
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Material Purchases" }
                Button { 
                    onclick: move |_| is_sheet_open.set(true),
                    "Add Purchase" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader { SheetTitle { "Add Material Purchase" } }
                    div { class: "flex flex-col gap-4 py-4 overflow-y-auto max-h-[70vh]",
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
                            Input { value: "{form_party}", oninput: move |e: FormEvent| form_party.set(e.value()) }
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
                        Button { onclick: submit_handler, "Save Purchase" }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for purchase in purchases.cloned().unwrap_or_default() {
                    Card { key: "{purchase.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center text-sm",
                                span { class: "text-muted-foreground", "{purchase.date}" }
                                div {
                                    class: if purchase.status == "PAID" {
                                        "bg-green-100 text-green-700 px-2 py-0.5 rounded-full text-xs font-semibold"
                                    } else {
                                        "bg-yellow-100 text-yellow-700 px-2 py-0.5 rounded-full text-xs font-semibold"
                                    },
                                    "{purchase.status}"
                                }
                            }
                        }
                        CardContent {
                            div { class: "flex flex-col gap-1",
                                div { class: "font-bold text-lg", "{purchase.material_name}" }
                                div { class: "text-sm text-muted-foreground flex justify-between",
                                    span { "{purchase.party_name}" }
                                    span { "{purchase.quantity_kg} KG @ ₹{purchase.rate_per_kg:.2}/KG" }
                                }
                            }
                        }
                        CardFooter { class: "bg-muted/50 pt-4 rounded-b-xl flex justify-between text-sm",
                            div { class: "flex flex-col",
                                span { class: "text-muted-foreground", "Advance" }
                                span { class: "font-medium text-green-600", "₹ {purchase.advance_paid:.2}" }
                            }
                            div { class: "flex flex-col items-end",
                                span { class: "text-muted-foreground", "Balance" }
                                span { class: "font-medium text-red-500", "₹ {purchase.balance:.2}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
