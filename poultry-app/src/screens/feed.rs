use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardHeader, CardFooter};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::*;
use crate::models::FeedBatch;

#[component]
pub fn Feed() -> Element {
    let mut batches = use_feed_batches();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut form_id = use_signal(|| Option::<String>::None);

    let mut form_date = use_signal(String::new);
    let mut form_batch_id = use_signal(String::new);
    let mut form_type = use_signal(String::new);
    let mut form_rate = use_signal(String::new);
    let mut form_total = use_signal(String::new);
    let mut form_payment = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let batch_id = form_batch_id().trim().to_string();
        let feed_type = form_type().trim().to_string();
        
        if date.is_empty() { form_error.set("Date required".to_string()); return; }
        if batch_id.is_empty() { form_error.set("Batch ID required".to_string()); return; }
        if feed_type.is_empty() { form_error.set("Feed Type required".to_string()); return; }
        
        let rate: f64 = match form_rate().parse() { Ok(v) => v, Err(_) => { form_error.set("Invalid rate".to_string()); return; } };
        let total: f64 = match form_total().parse() { Ok(v) => v, Err(_) => { form_error.set("Invalid total amount".to_string()); return; } };
        let payment: f64 = match form_payment().parse() { Ok(v) => v, Err(_) => { form_error.set("Invalid payment amount".to_string()); return; } };
        
        let new_record = FeedBatch {
            id: form_id().unwrap_or_default(),
            date,
            batch_id,
            feed_type,
            rate,
            total_amount: total,
            payment,
            opening_balance: 0.0,
            closing_balance: total - payment,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/feed/{}", id), &new_record).await
            } else {
                api.post("/api/feed", &new_record).await
            };
            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    batches.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Feed Mill" }
                Button { 
                    onclick: move |_| {
                        form_id.set(None);
                        form_date.set(String::new());
                        form_batch_id.set(String::new());
                        form_type.set(String::new());
                        form_rate.set(String::new());
                        form_total.set(String::new());
                        form_payment.set(String::new());
                        is_sheet_open.set(true);
                    }, 
                    "Add Feed Batch" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader { SheetTitle { if form_id().is_some() { "Edit Feed Batch" } else { "Add Feed Batch" } } }
                    div { class: "flex flex-col gap-4 px-6 py-4 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "date", "Date" }
                            Input { value: "{form_date}", oninput: move |e: FormEvent| form_date.set(e.value()), placeholder: "YYYY-MM-DD" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "batch", "Batch ID" }
                            Input { value: "{form_batch_id}", oninput: move |e: FormEvent| form_batch_id.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "type", "Feed Type" }
                            Input { value: "{form_type}", oninput: move |e: FormEvent| form_type.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "rate", "Rate" }
                            Input { value: "{form_rate}", oninput: move |e: FormEvent| form_rate.set(e.value()), placeholder: "0.00" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "total", "Total Amount" }
                            Input { value: "{form_total}", oninput: move |e: FormEvent| form_total.set(e.value()), placeholder: "0.00" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "payment", "Payment" }
                            Input { value: "{form_payment}", oninput: move |e: FormEvent| form_payment.set(e.value()), placeholder: "0.00" }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, if form_id().is_some() { "Update Batch" } else { "Save Batch" } }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for batch in batches.cloned().unwrap_or_default() {
                    Card { key: "{batch.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center",
                                span { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "{batch.date}" }
                                span { class: "px-2.5 py-0.5 rounded-full text-xs font-semibold bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 border border-stone-200 dark:border-stone-700", "Batch #{batch.batch_id}" }
                            }
                        }
                        CardContent {
                            div { class: "flex flex-col gap-3",
                                div { class: "flex justify-between items-baseline",
                                    span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{batch.feed_type}" }
                                    span { class: "text-base font-bold text-blue-600 dark:text-blue-400", "₹ {batch.total_amount:.2}" }
                                }
                                div { class: "grid grid-cols-3 gap-2 p-3 rounded-lg bg-stone-50 dark:bg-stone-800/60 border border-stone-200/60 dark:border-stone-800 text-xs",
                                    div { class: "flex flex-col",
                                        span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Rate" }
                                        span { class: "font-semibold text-gray-800 dark:text-gray-200", "₹{batch.rate:.2}" }
                                    }
                                    div { class: "flex flex-col",
                                        span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Payment" }
                                        span { class: "font-semibold text-green-600 dark:text-green-500", "₹{batch.payment:.2}" }
                                    }
                                    div { class: "flex flex-col items-end",
                                        span { class: "text-gray-500 dark:text-gray-400 mb-0.5", "Closing Bal." }
                                        span { class: "font-semibold text-blue-600 dark:text-blue-400", "₹{batch.closing_balance:.2}" }
                                    }
                                }
                            }
                        }
                        CardFooter {
                            div { class: "flex justify-end gap-2 w-full pt-1",
                                {
                                    let edit_batch = batch.clone();
                                    let delete_id = batch.id.clone();
                                    rsx! {
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            onclick: move |_| {
                                                form_id.set(Some(edit_batch.id.clone()));
                                                form_date.set(edit_batch.date.clone());
                                                form_batch_id.set(edit_batch.batch_id.clone());
                                                form_type.set(edit_batch.feed_type.clone());
                                                form_rate.set(edit_batch.rate.to_string());
                                                form_total.set(edit_batch.total_amount.to_string());
                                                form_payment.set(edit_batch.payment.to_string());
                                                is_sheet_open.set(true);
                                            },
                                            "Edit"
                                        }
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            onclick: move |_| {
                                                let id = delete_id.clone();
                                                spawn(async move {
                                                    let _ = api.delete(&format!("/api/feed/{}", id)).await;
                                                    batches.restart();
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
        }
    }
}
