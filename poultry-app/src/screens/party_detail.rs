use dioxus::prelude::*;
use crate::services::{use_party_ledger, use_parties};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};

#[component]
pub fn PartyDetail(id: String) -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    let parties = use_parties();
    let party_opt = parties.cloned().unwrap_or_default().into_iter().find(|p| p.id == id);
    let ledger = use_party_ledger(id.clone());

    let party_name = party_opt.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Loading...".to_string());
    
    // In a real system, the current_balance is the running balance calculated on the backend.
    // For this prototype, we'll start running balance at 0.0 and aggregate transactions,
    // since current_balance is meant to be the end result. 
    // Wait, let's just show the calculated balance row by row.
    let mut running_balance = 0.0;

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-4xl mx-auto pb-20",
            div { class: "flex items-center gap-4",
                Button { 
                    variant: ButtonVariant::Outline,
                    onclick: move |_| { nav.go_back(); },
                    "← Back" 
                }
                h1 { class: "text-2xl font-bold tracking-tight", "{party_name} - Ledger" }
            }

            Card { class: "mt-4",
                CardContent { class: "p-0 overflow-hidden rounded-xl",
                    div { class: "w-full overflow-x-auto",
                        table { class: "w-full text-sm text-left",
                            thead { class: "text-xs text-gray-500 uppercase bg-gray-50",
                                tr {
                                    th { class: "px-6 py-3", "Date" }
                                    th { class: "px-6 py-3", "Description" }
                                    th { class: "px-6 py-3 text-right", "Charge (Dr)" }
                                    th { class: "px-6 py-3 text-right", "Payment (Cr)" }
                                    th { class: "px-6 py-3 text-right", "Balance" }
                                }
                            }
                            tbody { class: "divide-y divide-border",
                                for entry in ledger.cloned().unwrap_or_default() {
                                    {
                                        running_balance = running_balance + entry.charge - entry.payment;
                                        rsx! {
                                            tr { class: "hover:bg-gray-50 transition-colors",
                                                td { class: "px-6 py-4 whitespace-nowrap text-gray-500", "{entry.date}" }
                                                td { class: "px-6 py-4", "{entry.description}" }
                                                td { class: "px-6 py-4 text-right text-red-500", 
                                                    if entry.charge > 0.0 { "₹ {entry.charge:.2}" } else { "-" }
                                                }
                                                td { class: "px-6 py-4 text-right text-green-600",
                                                    if entry.payment > 0.0 { "₹ {entry.payment:.2}" } else { "-" }
                                                }
                                                td { class: "px-6 py-4 text-right font-medium",
                                                    class: if running_balance < 0.0 { "text-red-500" } else { "text-green-600" },
                                                    "₹ {running_balance.abs():.2}"
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
    }
}
