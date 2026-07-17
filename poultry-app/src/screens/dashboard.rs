use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardHeader, CardTitle};
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    let mut auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_email".to_string(),
        || None::<String>,
    );
    let user_email = auth_email().unwrap_or_default();

    let logout_action = use_context::<crate::LogoutAction>();
    let on_signout = move |_| {
        nav.replace(crate::routes::AuthenticatedRoute::Dashboard {});
        logout_action.0.call(());
    };
    rsx! {
        div {
            class: "w-full flex flex-col gap-6 max-w-2xl mx-auto",

            // Header
            div {
                class: "w-full flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4",
                h1 { class: "text-2xl sm:text-3xl font-bold break-words", "🐔 Farmiz Dashboard" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: on_signout,
                    "Sign Out"
                }
            }

            // Welcome card
            Card {
                class: "bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200",
                CardContent {
                    class: "pt-6",
                    p { class: "text-gray-600 text-sm", "Welcome back," }
                    p { class: "text-lg font-semibold text-gray-900 mt-1", "{user_email}" }
                }
            }

            // Quick stats placeholder
            div {
                class: "grid grid-cols-1 sm:grid-cols-2 gap-4 w-full",
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-2xl sm:text-3xl font-bold text-green-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Active Flocks" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-2xl sm:text-3xl font-bold text-amber-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Total Birds" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-2xl sm:text-3xl font-bold text-blue-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Eggs Today" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-2xl sm:text-3xl font-bold text-purple-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Feed Stock (kg)" }
                    }
                }
            }

            // Getting started
            Card {
                CardHeader {
                    CardTitle { "🚀 Getting Started" }
                }
                CardContent {
                    p { class: "text-gray-600 text-sm",
                        "Your account is set up and ready to go. Dashboard features are coming soon!"
                    }
                }
            }
        }
    }
}
