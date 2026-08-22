use crate::components::card::{Card, CardContent, CardHeader, CardTitle};
use crate::i18n::tr;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let _nav = dioxus_router::hooks::use_navigator();
    let auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_email".to_string(),
        || None::<String>,
    );
    let user_email = auth_email().unwrap_or_default();

    let stats_resource = crate::services::use_dashboard_stats();
    let stats = stats_resource.cloned().and_then(|r| r.ok());
    let today_sales = stats.as_ref().map(|s| s.today_sales).unwrap_or(0.0);
    let eggs_sold = stats.as_ref().map(|s| s.eggs_sold_today).unwrap_or(0);
    let today_purchases = stats.as_ref().map(|s| s.today_purchases).unwrap_or(0.0);
    let active_parties = stats.as_ref().map(|s| s.active_parties).unwrap_or(0);

    let dash_title = tr("dashboard");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",
            // FIXED HEADER
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-1",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "🏠 {dash_title}" }
                p { class: "text-xs text-gray-500 dark:text-gray-400", "Farm operational summary, today's business metrics & quick overview" }
            }

            // SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-6 w-full max-w-5xl mx-auto pb-20",
                    // Welcome card
                    Card {
                        class: "bg-gradient-to-r from-blue-50/80 to-indigo-50/80 border-blue-200 dark:from-stone-800/80 dark:to-stone-850/80 dark:border-stone-700",
                        CardContent {
                            p { class: "text-gray-600 dark:text-gray-400 text-sm", "Welcome back," }
                            p { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mt-1", "{user_email}" }
                        }
                    }

                    // Quick stats
                    div {
                        class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 w-full",
                        Card {
                            CardContent {
                                p { class: "text-2xl sm:text-3xl font-bold text-green-600 dark:text-green-500", "₹ {today_sales:.2}" }
                                p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Today's Sales" }
                            }
                        }
                        Card {
                            CardContent {
                                p { class: "text-2xl sm:text-3xl font-bold text-amber-600 dark:text-amber-500", "{eggs_sold}" }
                                p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Eggs Sold Today" }
                            }
                        }
                        Card {
                            CardContent {
                                p { class: "text-2xl sm:text-3xl font-bold text-blue-600 dark:text-blue-500", "₹ {today_purchases:.2}" }
                                p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Today's Purchases" }
                            }
                        }
                        Card {
                            CardContent {
                                p { class: "text-2xl sm:text-3xl font-bold text-purple-600 dark:text-purple-500", "{active_parties}" }
                                p { class: "text-sm text-gray-500 dark:text-gray-400 mt-1", "Active Parties" }
                            }
                        }
                    }

                    // Getting started
                    Card {
                        CardHeader {
                            CardTitle { "🚀 Getting Started" }
                        }
                        CardContent {
                            p { class: "text-gray-600 dark:text-gray-300 text-sm",
                                "Your farm management account is ready. Use the sidebar navigation to manage daily egg production, sales records, material purchases, feed milling, and labor payroll."
                            }
                        }
                    }
                }
            }
        }
    }
}
