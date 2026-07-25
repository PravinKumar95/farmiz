use dioxus::prelude::*;
use chrono::Utc;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::DailyProduction;
use crate::services::*;

#[component]
pub fn Production() -> Element {
    let mut logs = use_daily_production();
    let api = use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let today_str = Utc::now().format("%Y-%m-%d").to_string();

    let mut form_date = use_signal(move || today_str.clone());
    let mut form_shed = use_signal(String::new);
    let mut form_good = use_signal(|| "0".to_string());
    let mut form_damaged = use_signal(|| "0".to_string());
    let mut form_mortality = use_signal(|| "0".to_string());
    let mut form_cull = use_signal(|| "0".to_string());
    let mut form_feed = use_signal(|| "0.0".to_string());
    let mut form_notes = use_signal(String::new);
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let shed_name = form_shed().trim().to_string();

        if date.is_empty() {
            form_error.set("Date is required.".to_string());
            return;
        }
        if shed_name.is_empty() {
            form_error.set("Shed name is required.".to_string());
            return;
        }

        let good: i32 = match form_good().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid good eggs count.".to_string());
                return;
            }
        };
        let damaged: i32 = match form_damaged().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid damaged eggs count.".to_string());
                return;
            }
        };
        let mortality: i32 = match form_mortality().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid mortality count.".to_string());
                return;
            }
        };
        let cull: i32 = match form_cull().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid cull count.".to_string());
                return;
            }
        };
        let feed: f64 = match form_feed().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid feed consumed.".to_string());
                return;
            }
        };

        let notes_val = if form_notes().trim().is_empty() {
            None
        } else {
            Some(form_notes().trim().to_string())
        };

        let record = DailyProduction {
            id: form_id().unwrap_or_default(),
            date,
            shed_name,
            egg_count_good: good,
            egg_count_damaged: damaged,
            mortality_count: mortality,
            cull_count: cull,
            feed_consumed_kg: feed,
            notes: notes_val,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/production/{}", id), &record).await
            } else {
                api.post("/api/production", &record).await
            };

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    logs.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    let confirm_delete = move |_| {
        if let Some(id) = delete_id() {
            spawn(async move {
                if let Ok(_) = api.delete(&format!("/api/production/{}", id)).await {
                    delete_id.set(None);
                    logs.restart();
                }
            });
        }
    };

    let records_list = logs.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let total_good_today: i32 = records_list.iter().map(|r| r.egg_count_good).sum();
    let total_damaged_today: i32 = records_list.iter().map(|r| r.egg_count_damaged).sum();
    let total_mortality_today: i32 = records_list.iter().map(|r| r.mortality_count).sum();
    let total_feed_today: f64 = records_list.iter().map(|r| r.feed_consumed_kg).sum();

    rsx! {
        div { class: "flex flex-col gap-6 w-full max-w-6xl mx-auto pb-20",
            div { class: "flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4",
                div {
                    h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "🥚 Daily Production & Flock Health Log" }
                    p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Track daily egg collection, damaged eggs, mortality, and feed consumption per shed." }
                }
                Button {
                    onclick: move |_| {
                        form_id.set(None);
                        form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                        form_shed.set(String::new());
                        form_good.set("0".to_string());
                        form_damaged.set("0".to_string());
                        form_mortality.set("0".to_string());
                        form_cull.set("0".to_string());
                        form_feed.set("0.0".to_string());
                        form_notes.set(String::new());
                        form_error.set(String::new());
                        is_sheet_open.set(true);
                    },
                    "+ Log Production"
                }
            }

            div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4 w-full",
                Card {
                    CardContent {
                        p { class: "text-2xl font-bold text-green-600 dark:text-green-400", "{total_good_today}" }
                        p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Good Eggs Collected" }
                    }
                }
                Card {
                    CardContent {
                        p { class: "text-2xl font-bold text-amber-600 dark:text-amber-400", "{total_damaged_today}" }
                        p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Damaged / Crates Eggs" }
                    }
                }
                Card {
                    CardContent {
                        p { class: "text-2xl font-bold text-red-600 dark:text-red-400", "{total_mortality_today}" }
                        p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Total Mortality Count" }
                    }
                }
                Card {
                    CardContent {
                        p { class: "text-2xl font-bold text-blue-600 dark:text-blue-400", "{total_feed_today:.1} kg" }
                        p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Feed Consumed" }
                    }
                }
            }

            match logs.cloned() {
                Some(Ok(items)) if !items.is_empty() => rsx! {
                    Card {
                        div { class: "overflow-x-auto",
                            table { class: "w-full text-sm text-left",
                                thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-border",
                                    tr {
                                        th { class: "px-6 py-3", "Date" }
                                        th { class: "px-6 py-3", "Shed" }
                                        th { class: "px-6 py-3 text-right", "Good Eggs" }
                                        th { class: "px-6 py-3 text-right", "Damaged" }
                                        th { class: "px-6 py-3 text-right", "Mortality" }
                                        th { class: "px-6 py-3 text-right", "Feed (kg)" }
                                        th { class: "px-6 py-3 text-right", "Actions" }
                                    }
                                }
                                tbody { class: "divide-y divide-border",
                                    for item in items {
                                        tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800 transition-colors",
                                            td { class: "px-6 py-4 font-medium text-gray-900 dark:text-gray-100", "{item.date}" }
                                            td { class: "px-6 py-4 text-gray-600 dark:text-gray-300", "{item.shed_name}" }
                                            td { class: "px-6 py-4 text-right font-semibold text-green-600 dark:text-green-400", "{item.egg_count_good}" }
                                            td { class: "px-6 py-4 text-right text-amber-600 dark:text-amber-400", "{item.egg_count_damaged}" }
                                            td { class: "px-6 py-4 text-right text-red-600 dark:text-red-400", "{item.mortality_count}" }
                                            td { class: "px-6 py-4 text-right text-blue-600 dark:text-blue-400", "{item.feed_consumed_kg:.1}" }
                                            td { class: "px-6 py-4 text-right flex items-center justify-end gap-2",
                                                {
                                                    let edit_item = item.clone();
                                                    let del_id = item.id.clone();
                                                    rsx! {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                form_id.set(Some(edit_item.id.clone()));
                                                                form_date.set(edit_item.date.clone());
                                                                form_shed.set(edit_item.shed_name.clone());
                                                                form_good.set(edit_item.egg_count_good.to_string());
                                                                form_damaged.set(edit_item.egg_count_damaged.to_string());
                                                                form_mortality.set(edit_item.mortality_count.to_string());
                                                                form_cull.set(edit_item.cull_count.to_string());
                                                                form_feed.set(edit_item.feed_consumed_kg.to_string());
                                                                form_notes.set(edit_item.notes.clone().unwrap_or_default());
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
                        }
                    }
                },
                Some(Ok(_)) => rsx! {
                    EmptyState {
                        icon: "🥚",
                        title: "No production records found",
                        description: "Log your daily egg collection, mortality, and feed metrics."
                    }
                },
                Some(Err(err)) => rsx! {
                    ErrorState { message: err }
                },
                None => rsx! { LoadingState {} }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle {
                            if form_id().is_some() { "Edit Production Record" } else { "Log Daily Production" }
                        }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "prod-date", "Date" }
                            Input { r#type: "date", value: "{form_date}", oninput: move |e: Event<FormData>| form_date.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "prod-shed", "Shed Name" }
                            Input { placeholder: "e.g. Shed A, Shed 1", value: "{form_shed}", oninput: move |e: Event<FormData>| form_shed.set(e.value()) }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "prod-good", "Good Eggs" }
                                Input { r#type: "number", value: "{form_good}", oninput: move |e: Event<FormData>| form_good.set(e.value()) }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "prod-damaged", "Damaged Eggs" }
                                Input { r#type: "number", value: "{form_damaged}", oninput: move |e: Event<FormData>| form_damaged.set(e.value()) }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "prod-mortality", "Mortality Count" }
                                Input { r#type: "number", value: "{form_mortality}", oninput: move |e: Event<FormData>| form_mortality.set(e.value()) }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "prod-cull", "Cull Count" }
                                Input { r#type: "number", value: "{form_cull}", oninput: move |e: Event<FormData>| form_cull.set(e.value()) }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "prod-feed", "Feed Consumed (KG)" }
                            Input { r#type: "number", step: "0.1", value: "{form_feed}", oninput: move |e: Event<FormData>| form_feed.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "prod-notes", "Notes (Optional)" }
                            Input { placeholder: "Vaccination, temperature, etc.", value: "{form_notes}", oninput: move |e: Event<FormData>| form_notes.set(e.value()) }
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
                            "Save Record"
                        }
                    }
                }
            }

            ConfirmDialog {
                is_open: delete_id().is_some(),
                title: "Delete Production Record".to_string(),
                description: "Are you sure you want to delete this production record? This action cannot be undone.".to_string(),
                onconfirm: confirm_delete,
                oncancel: move |_| delete_id.set(None)
            }
        }
    }
}
