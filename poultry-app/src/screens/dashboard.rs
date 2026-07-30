use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardHeader, CardTitle};
use crate::i18n::tr;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    let auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_email".to_string(),
        || None::<String>,
    );
    let user_email = auth_email().unwrap_or_default();

    let logout_action = use_context::<crate::LogoutAction>();
    let on_signout = move |_| {
        nav.replace(crate::routes::AuthenticatedRoute::Dashboard {});
        logout_action.0.call(());
    };

    let stats_resource = crate::services::use_dashboard_stats();
    let stats = stats_resource.cloned().and_then(|r| r.ok());
    let today_sales = stats.as_ref().map(|s| s.today_sales).unwrap_or(0.0);
    let eggs_sold = stats.as_ref().map(|s| s.eggs_sold_today).unwrap_or(0);
    let today_purchases = stats.as_ref().map(|s| s.today_purchases).unwrap_or(0.0);
    let active_parties = stats.as_ref().map(|s| s.active_parties).unwrap_or(0);

    let dash_title = tr("dashboard");
    let signout_lbl = tr("sign-out");

    rsx! {
        div {
            class: "w-full flex flex-col gap-6 max-w-2xl mx-auto p-4 md:p-6",

            // Header
            div {
                class: "w-full flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4",
                h1 { class: "text-2xl sm:text-3xl font-bold break-words text-gray-900 dark:text-gray-100", "🏠 {dash_title}" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: on_signout,
                    "{signout_lbl}"
                }
            }

            // Welcome card
            Card {
                class: "bg-gradient-to-r from-blue-50/80 to-indigo-50/80 border-blue-200 dark:from-stone-800/80 dark:to-stone-850/80 dark:border-stone-700",
                CardContent {
                    p { class: "text-gray-600 dark:text-gray-400 text-sm", "Welcome back," }
                    p { class: "text-lg font-semibold text-gray-900 dark:text-gray-100 mt-1", "{user_email}" }
                }
            }

            // Quick stats placeholder
            div {
                class: "grid grid-cols-1 sm:grid-cols-2 gap-4 w-full",
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
                        "Your account is set up and ready to go. Dashboard features are coming soon!"
                    }
                }
            }
        }
    }
}
