use dioxus::prelude::*;
use dioxus_router::components::Link;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::Party;
use crate::services::*;

#[component]
pub fn Parties() -> Element {
    let mut parties = use_parties();
    let api = use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let mut form_name = use_signal(String::new);
    let mut form_type = use_signal(String::new);
    let mut form_balance = use_signal(|| "0.0".to_string());
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let name = form_name().trim().to_string();
        let p_type = form_type().trim().to_uppercase();
        let balance_str = form_balance().trim().to_string();

        if name.len() < 2 {
            form_error.set("Name must be at least 2 characters long.".to_string());
            return;
        }
        let allowed_types = ["CUSTOMER", "SUPPLIER", "EMPLOYEE", "BAKERY"];
        if !allowed_types.contains(&p_type.as_str()) {
            form_error.set("Type must be CUSTOMER, SUPPLIER, EMPLOYEE, or BAKERY.".to_string());
            return;
        }
        let balance: f64 = match balance_str.parse() {
            Ok(b) => b,
            Err(_) => {
                form_error.set("Balance must be a valid number.".to_string());
                return;
            }
        };

        form_error.set(String::new());
        let party = Party {
            id: form_id().unwrap_or_default(),
            name,
            party_type: p_type,
            current_balance: balance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/parties/{}", id), &party).await
            } else {
                api.post("/api/parties", &party).await
            };

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_name.set(String::new());
                    form_type.set(String::new());
                    form_balance.set("0.0".to_string());
                    parties.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    let confirm_delete = move |_| {
        if let Some(id) = delete_id() {
            spawn(async move {
                if let Ok(_) = api.delete(&format!("/api/parties/{}", id)).await {
                    delete_id.set(None);
                    parties.restart();
                }
            });
        }
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-4xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Parties Directory" }
                Button {
                    onclick: move |_| {
                        form_id.set(None);
                        form_name.set(String::new());
                        form_type.set("CUSTOMER".to_string());
                        form_balance.set("0.0".to_string());
                        form_error.set(String::new());
                        is_sheet_open.set(true);
                    },
                    "Add Party"
                }
            }

            match parties.cloned() {
                Some(Ok(list)) if !list.is_empty() => rsx! {
                    Card { class: "mt-4",
                        CardContent { class: "p-0 overflow-hidden rounded-xl",
                            div { class: "overflow-x-auto",
                                table { class: "w-full text-sm text-left",
                                    thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-border",
                                        tr {
                                            th { class: "px-6 py-3", "Party Name" }
                                            th { class: "px-6 py-3", "Type" }
                                            th { class: "px-6 py-3 text-right", "Current Balance" }
                                            th { class: "px-6 py-3 text-right", "Actions" }
                                        }
                                    }
                                    tbody { class: "divide-y divide-border",
                                        for party in list {
                                            tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800 transition-colors",
                                                td { class: "px-6 py-4 font-medium text-gray-900 dark:text-gray-100",
                                                    Link {
                                                        to: crate::routes::AuthenticatedRoute::PartyDetail { id: party.id.clone() },
                                                        class: "hover:underline text-blue-600 dark:text-blue-400 font-semibold",
                                                        "{party.name}"
                                                    }
                                                }
                                                td { class: "px-6 py-4 text-xs font-semibold text-gray-600 dark:text-gray-300", "{party.party_type}" }
                                                td { class: "px-6 py-4 text-right font-bold text-gray-900 dark:text-gray-100", "₹ {party.current_balance:.2}" }
                                                td { class: "px-6 py-4 text-right flex items-center justify-end gap-2",
                                                    {
                                                        let p_edit = party.clone();
                                                        let p_del_id = party.id.clone();
                                                        rsx! {
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                onclick: move |_| {
                                                                    form_id.set(Some(p_edit.id.clone()));
                                                                    form_name.set(p_edit.name.clone());
                                                                    form_type.set(p_edit.party_type.clone());
                                                                    form_balance.set(p_edit.current_balance.to_string());
                                                                    is_sheet_open.set(true);
                                                                },
                                                                "Edit"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: move |_| {
                                                                    delete_id.set(Some(p_del_id.clone()));
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
                },
                Some(Ok(_)) => rsx! {
                    EmptyState {
                        icon: "📒",
                        title: "No Parties Registered",
                        description: "No customers, suppliers, or bakeries registered yet."
                    }
                },
                Some(Err(err)) => rsx! { ErrorState { message: err } },
                None => rsx! { LoadingState {} }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if form_id().is_some() { "Edit Party" } else { "Add New Party" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-name", "Party Name" }
                            Input {
                                placeholder: "e.g. John Doe, Fresh Bakery",
                                value: "{form_name}",
                                oninput: move |e: Event<FormData>| form_name.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-type", "Party Type" }
                            select {
                                class: "w-full flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm dark:bg-stone-800 dark:border-stone-700 dark:text-gray-100",
                                value: "{form_type}",
                                onchange: move |e: Event<FormData>| form_type.set(e.value()),
                                option { value: "CUSTOMER", "CUSTOMER" }
                                option { value: "SUPPLIER", "SUPPLIER" }
                                option { value: "BAKERY", "BAKERY" }
                                option { value: "EMPLOYEE", "EMPLOYEE" }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-balance", "Initial Balance (₹)" }
                            Input {
                                r#type: "number",
                                step: "0.01",
                                value: "{form_balance}",
                                oninput: move |e: Event<FormData>| form_balance.set(e.value())
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
                            if form_id().is_some() { "Update Party" } else { "Save Party" }
                        }
                    }
                }
            }

            ConfirmDialog {
                is_open: delete_id().is_some(),
                title: "Delete Party".to_string(),
                description: "Are you sure you want to delete this party? All associated transaction histories will be unlinked.".to_string(),
                onconfirm: confirm_delete,
                oncancel: move |_| delete_id.set(None)
            }
        }
    }
}
