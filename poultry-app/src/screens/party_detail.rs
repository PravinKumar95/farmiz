use dioxus::prelude::*;
use crate::services::{use_party_ledger, use_parties};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};
use crate::components::empty_state::{EmptyState, LoadingState, ErrorState};

#[component]
pub fn PartyDetail(id: String) -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    let parties = use_parties();
    let party_opt = parties.cloned().and_then(|r| r.ok()).unwrap_or_default().into_iter().find(|p| p.id == id);
    let mut ledger = use_party_ledger(id.clone());

    let party_name = party_opt.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Loading...".to_string());
    let party_type = party_opt.as_ref().map(|p| p.party_type.clone()).unwrap_or_else(|| "CUSTOMER".to_string());
    let is_supplier = party_type == "SUPPLIER";

    let mut running_balance = 0.0;

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-4xl mx-auto pb-20 p-4 md:p-6",
            div { class: "flex items-center justify-between gap-4 border-b border-stone-200 dark:border-stone-800 pb-3",
                div { class: "flex items-center gap-3",
                    Button { 
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.go_back(); },
                        "← Back" 
                    }
                    div {
                        h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "{party_name}" }
                        p { class: "text-xs text-gray-500 dark:text-gray-400 mt-0.5", 
                            span { class: "px-2 py-0.5 rounded text-xs font-semibold mr-2 bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300", "{party_type}" }
                            "Transaction statement & running ledger balance"
                        }
                    }
                }
            }

            Card { class: "mt-2",
                CardContent { class: "p-0 overflow-hidden rounded-xl",
                    match ledger.cloned() {
                        Some(Ok(entries)) if !entries.is_empty() => rsx! {
                            div { class: "overflow-x-auto",
                                table { class: "w-full text-sm text-left",
                                    thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-border",
                                        tr {
                                            th { class: "px-6 py-3", "Date" }
                                            th { class: "px-6 py-3", "Description" }
                                            th { class: "px-6 py-3 text-right", "Charge (Dr)" }
                                            th { class: "px-6 py-3 text-right", "Payment (Cr)" }
                                            th { class: "px-6 py-3 text-right", "Running Balance" }
                                        }
                                    }
                                    tbody { class: "divide-y divide-border",
                                        for entry in entries {
                                            {
                                                if is_supplier {
                                                    // For suppliers, payment is cash paid out, charge is invoice amount
                                                    running_balance = running_balance + entry.payment - entry.charge;
                                                } else {
                                                    // For customers/bakeries, charge is sale amount, payment is cash received
                                                    running_balance = running_balance + entry.charge - entry.payment;
                                                }
                                                let is_favorable = running_balance >= 0.0;
                                                let bal_str = if is_supplier {
                                                    if running_balance < 0.0 {
                                                        format!("-₹ {:.2} (Payable)", running_balance.abs())
                                                    } else if running_balance > 0.0 {
                                                        format!("+₹ {:.2} (Surplus)", running_balance)
                                                    } else {
                                                        "₹ 0.00".to_string()
                                                    }
                                                } else {
                                                    if running_balance > 0.0 {
                                                        format!("₹ {:.2} (Due)", running_balance)
                                                    } else if running_balance < 0.0 {
                                                        format!("-₹ {:.2} (Overpaid)", running_balance.abs())
                                                    } else {
                                                        "₹ 0.00".to_string()
                                                    }
                                                };
                                                rsx! {
                                                    tr { class: "hover:bg-gray-50 dark:bg-stone-800 transition-colors",
                                                        td { class: "px-6 py-4 whitespace-nowrap text-gray-500 dark:text-gray-400 font-mono text-xs", "{entry.date}" }
                                                        td { class: "px-6 py-4 font-medium text-gray-900 dark:text-gray-100", "{entry.description}" }
                                                        td { class: "px-6 py-4 text-right text-red-500 font-medium", 
                                                            if entry.charge > 0.0 { "₹ {entry.charge:.2}" } else { "-" }
                                                        }
                                                        td { class: "px-6 py-4 text-right text-emerald-600 dark:text-emerald-400 font-medium",
                                                            if entry.payment > 0.0 { "₹ {entry.payment:.2}" } else { "-" }
                                                        }
                                                        td { class: "px-6 py-4 text-right font-bold",
                                                            class: if is_favorable { "text-emerald-600 dark:text-emerald-400" } else { "text-rose-600 dark:text-rose-400" },
                                                            "{bal_str}"
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
                                icon: "📖",
                                title: "No Ledger Entries",
                                description: "No transaction records found for this party ledger."
                            }
                        },
                        Some(Err(err)) => rsx! {
                            ErrorState {
                                message: err,
                                on_retry: move |_| { ledger.restart(); }
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
