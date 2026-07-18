use dioxus::prelude::*;
use dioxus_router::components::Link;
use crate::components::button::Button;
use crate::components::card::{Card, CardContent};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::services::*;
use crate::models::Party;

#[component]
pub fn Parties() -> Element {
    let mut parties = use_parties();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);
    
    // Form state
    let mut form_name = use_signal(String::new);
    let mut form_type = use_signal(String::new);
    let mut form_balance = use_signal(String::new);
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
        let new_party = Party {
            id: String::new(),
            name,
            party_type: p_type,
            current_balance: balance,
            created_at: None,
        };

        spawn(async move {
            match api.post("/api/parties", &new_party).await {
                Ok(_) => {

                    is_sheet_open.set(false);
                    form_name.set(String::new());
                    form_type.set(String::new());
                    form_balance.set(String::new());
                    parties.restart();
                }
                Err(e) => {
                    form_error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Parties Directory" }
                Button { 
                    onclick: move |_| is_sheet_open.set(true),
                    "Add Party" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { "Add New Party" }
                    }
                    div { class: "flex flex-col gap-4 py-4",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "name", "Name" }
                            Input {
                                value: "{form_name}",
                                oninput: move |e: FormEvent| form_name.set(e.value()),
                                placeholder: "e.g., Rajendar"
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "type", "Type" }
                            Input {
                                value: "{form_type}",
                                oninput: move |e: FormEvent| form_type.set(e.value()),
                                placeholder: "CUSTOMER / SUPPLIER / EMPLOYEE / BAKERY"
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "balance", "Opening Balance" }
                            Input {
                                value: "{form_balance}",
                                oninput: move |e: FormEvent| form_balance.set(e.value()),
                                placeholder: "0.00"
                            }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, "Save Party" }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for party in parties.cloned().unwrap_or_default() {
                    Link {
                        key: "{party.id}",
                        to: crate::routes::AuthenticatedRoute::PartyDetail { id: party.id.clone() },
                        class: "block transition-transform hover:scale-[1.01] hover:shadow-md rounded-xl",
                        Card {
                            CardContent {
                                div { class: "flex justify-between items-center",
                                    div { class: "flex flex-col",
                                        span { class: "font-bold text-base", "{party.name}" }
                                        span { class: "text-xs text-gray-500 mt-1 uppercase tracking-wider", "{party.party_type}" }
                                    }
                                    div { class: "flex flex-col items-end",
                                        div { class: "text-xs text-gray-500 mb-1", "Current Balance" }
                                        div {
                                            class: if party.current_balance < 0.0 {
                                                "font-bold text-lg text-red-500"
                                            } else {
                                                "font-bold text-lg text-green-600"
                                            },
                                            "₹ {party.current_balance.abs():.2}"
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
