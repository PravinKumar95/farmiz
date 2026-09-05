use dioxus::prelude::*;
use crate::services::{use_party_ledger, use_parties};
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};
use crate::components::date_range_filter::{DateRange, DateRangeFilter};
use crate::components::layout_toggle::{LayoutToggle, ViewLayout};
use crate::components::empty_state::{EmptyState, LoadingState, ErrorState};

fn format_balance_display(bal: f64, is_supplier: bool) -> (bool, String) {
    let is_favorable = bal >= 0.0;
    let bal_str = if is_supplier {
        if bal < -0.005 {
            format!("-₹ {:.2} (Payable)", bal.abs())
        } else if bal > 0.005 {
            format!("+₹ {:.2} (Surplus)", bal)
        } else {
            "₹ 0.00".to_string()
        }
    } else {
        if bal > 0.005 {
            format!("₹ {:.2} (Due)", bal)
        } else if bal < -0.005 {
            format!("-₹ {:.2} (Overpaid)", bal.abs())
        } else {
            "₹ 0.00".to_string()
        }
    };
    (is_favorable, bal_str)
}

#[component]
pub fn PartyDetail(id: String) -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    let parties = use_parties();
    let party_opt = parties.cloned().and_then(|r| r.ok()).unwrap_or_default().into_iter().find(|p| p.id == id);
    let mut ledger = use_party_ledger(id.clone());

    let mut view_layout = use_signal(ViewLayout::default);
    let mut date_range = use_signal(DateRange::default);

    let party_name = party_opt.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Loading...".to_string());
    let party_type = party_opt.as_ref().map(|p| p.party_type.clone()).unwrap_or_else(|| "CUSTOMER".to_string());
    let is_supplier = party_type == "SUPPLIER";

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",
            // FIXED HEADER
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex items-center justify-between gap-2",
                    div { class: "flex items-center gap-2 min-w-0",
                        Button { 
                            variant: ButtonVariant::Outline,
                            onclick: move |_| { nav.go_back(); },
                            "← Back" 
                        }
                        div { class: "min-w-0",
                            h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "{party_name}" }
                            p { class: "hidden sm:flex text-xs text-gray-500 dark:text-gray-400 mt-0.5 items-center gap-2", 
                                span { class: "px-2 py-0.5 rounded text-xs font-semibold bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300", "{party_type}" }
                                span { "Transaction statement & running ledger balance" }
                            }
                        }
                    }
                    LayoutToggle {
                        selected: view_layout(),
                        onchange: move |vl| view_layout.set(vl)
                    }
                }
                DateRangeFilter {
                    selected: date_range(),
                    onchange: move |dr| date_range.set(dr)
                }
            }

            // SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-3 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-16 md:pb-20",
                    match ledger.cloned() {
                        Some(Ok(mut entries)) => {
                            // Ensure strict chronological ordering
                            entries.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.created_at.cmp(&b.created_at)));

                            let mut opening_balance = 0.0;
                            let mut prior_entries_count = 0;
                            let mut filtered_entries = Vec::new();

                            for entry in entries.into_iter() {
                                if date_range().is_before(&entry.date) {
                                    if is_supplier {
                                        opening_balance = opening_balance + entry.payment - entry.charge;
                                    } else {
                                        opening_balance = opening_balance + entry.charge - entry.payment;
                                    }
                                    prior_entries_count += 1;
                                } else if date_range().matches(&entry.date) {
                                    filtered_entries.push(entry);
                                }
                            }

                            let show_opening_balance = !date_range().is_all() || prior_entries_count > 0;

                            if !filtered_entries.is_empty() || (show_opening_balance && prior_entries_count > 0) {
                                if view_layout() == ViewLayout::Table {
                                    let mut running_balance = opening_balance;
                                    rsx! {
                                        Card {
                                            CardContent { class: "p-0 overflow-hidden rounded-xl",
                                                div { class: "overflow-x-auto",
                                                    table { class: "w-full text-sm text-left",
                                                        thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                            tr {
                                                                th { class: "px-6 py-3", "Date" }
                                                                th { class: "px-6 py-3", "Description" }
                                                                th { class: "px-6 py-3 text-right", "Charge (Dr)" }
                                                                th { class: "px-6 py-3 text-right", "Payment (Cr)" }
                                                                th { class: "px-6 py-3 text-right", "Running Balance" }
                                                            }
                                                        }
                                                        tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                            if show_opening_balance {
                                                                {
                                                                    let (is_fav, bal_str) = format_balance_display(opening_balance, is_supplier);
                                                                    rsx! {
                                                                        tr { class: "bg-amber-50/50 dark:bg-amber-950/20 font-medium italic border-b border-amber-200/50 dark:border-amber-900/30",
                                                                            td { class: "px-6 py-4 whitespace-nowrap text-stone-400 font-mono text-xs", "—" }
                                                                            td { class: "px-6 py-4 font-semibold text-stone-800 dark:text-stone-200 flex items-center gap-1.5",
                                                                                span { "💼" }
                                                                                span { "Opening Balance (Brought Forward)" }
                                                                            }
                                                                            td { class: "px-6 py-4 text-right text-stone-400", "—" }
                                                                            td { class: "px-6 py-4 text-right text-stone-400", "—" }
                                                                            td { class: "px-6 py-4 text-right font-bold",
                                                                                class: if is_fav { "text-emerald-600 dark:text-emerald-400" } else { "text-rose-600 dark:text-rose-400" },
                                                                                "{bal_str}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            if filtered_entries.is_empty() {
                                                                tr {
                                                                    td { colspan: "5", class: "px-6 py-8 text-center text-sm text-stone-500 dark:text-stone-400 italic",
                                                                        "No transaction records in this date range."
                                                                    }
                                                                }
                                                            }
                                                            for entry in filtered_entries {
                                                                {
                                                                    if is_supplier {
                                                                        // For suppliers, payment is cash paid out, charge is invoice amount
                                                                        running_balance = running_balance + entry.payment - entry.charge;
                                                                    } else {
                                                                        // For customers/bakeries, charge is sale amount, payment is cash received
                                                                        running_balance = running_balance + entry.charge - entry.payment;
                                                                    }
                                                                    let (is_favorable, bal_str) = format_balance_display(running_balance, is_supplier);
                                                                    rsx! {
                                                                        tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
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
                                            }
                                        }
                                    }
                                } else {
                                    let mut running_balance = opening_balance;
                                    rsx! {
                                        div { class: "flex flex-col gap-3",
                                            if show_opening_balance {
                                                {
                                                    let (is_fav, bal_str) = format_balance_display(opening_balance, is_supplier);
                                                    rsx! {
                                                        Card {
                                                            div { class: "p-4 flex flex-col gap-2 bg-amber-50/50 dark:bg-amber-950/20 border-l-4 border-amber-500 rounded-lg",
                                                                div { class: "flex justify-between items-center",
                                                                    span { class: "text-xs font-bold text-amber-800 dark:text-amber-400 uppercase tracking-wider flex items-center gap-1",
                                                                        "💼 Opening Balance (Brought Forward)"
                                                                    }
                                                                    span {
                                                                        class: if is_fav { "px-2.5 py-0.5 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300" } else { "px-2.5 py-0.5 rounded-full text-xs font-bold bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-300" },
                                                                        "{bal_str}"
                                                                    }
                                                                }
                                                                p { class: "text-xs text-stone-500 dark:text-stone-400 italic", "Brought forward balance prior to selected period" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if filtered_entries.is_empty() {
                                                div { class: "p-6 text-center text-sm text-stone-500 dark:text-stone-400 italic bg-stone-50 dark:bg-stone-800/40 rounded-xl",
                                                    "No transaction records in this date range."
                                                }
                                            }
                                            for entry in filtered_entries {
                                                {
                                                    if is_supplier {
                                                        running_balance = running_balance + entry.payment - entry.charge;
                                                    } else {
                                                        running_balance = running_balance + entry.charge - entry.payment;
                                                    }
                                                    let (is_favorable, bal_str) = format_balance_display(running_balance, is_supplier);
                                                    rsx! {
                                                        Card { key: "{entry.date}_{entry.description}",
                                                            div { class: "p-4 flex flex-col gap-2.5",
                                                                div { class: "flex justify-between items-center",
                                                                    span { class: "text-xs font-mono text-gray-500 dark:text-gray-400", "🗓️ {entry.date}" }
                                                                    span {
                                                                        class: if is_favorable { "px-2.5 py-0.5 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300" } else { "px-2.5 py-0.5 rounded-full text-xs font-bold bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-300" },
                                                                        "Balance: {bal_str}"
                                                                    }
                                                                }
                                                                p { class: "text-sm font-semibold text-gray-900 dark:text-gray-100", "{entry.description}" }
                                                                div { class: "grid grid-cols-2 gap-2 pt-2 border-t border-stone-100 dark:border-stone-800 text-xs",
                                                                    div {
                                                                        span { class: "text-gray-500 dark:text-gray-400", "Charge (Dr): " }
                                                                        span { class: "font-semibold text-red-500",
                                                                            if entry.charge > 0.0 { "₹ {entry.charge:.2}" } else { "-" }
                                                                        }
                                                                    }
                                                                    div { class: "text-right",
                                                                        span { class: "text-gray-500 dark:text-gray-400", "Payment (Cr): " }
                                                                        span { class: "font-semibold text-emerald-600 dark:text-emerald-400",
                                                                            if entry.payment > 0.0 { "₹ {entry.payment:.2}" } else { "-" }
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
                            } else {
                                rsx! {
                                    EmptyState {
                                        icon: "📖",
                                        title: "No Ledger Entries",
                                        description: "No transaction records match your selected date range."
                                    }
                                }
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
