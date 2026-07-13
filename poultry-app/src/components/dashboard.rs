use dioxus::prelude::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardHeader, CardTitle};

#[component]
pub fn Dashboard(user_email: String, on_signout: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-6 p-6 max-w-2xl mx-auto",

            // Header
            div {
                class: "flex items-center justify-between",
                h1 { class: "text-3xl font-bold", "🐔 Farmiz Dashboard" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| on_signout.call(()),
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
                class: "grid grid-cols-2 gap-4",
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-3xl font-bold text-green-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Active Flocks" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-3xl font-bold text-amber-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Total Birds" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-3xl font-bold text-blue-600", "—" }
                        p { class: "text-sm text-gray-500 mt-1", "Eggs Today" }
                    }
                }
                Card {
                    CardContent {
                        class: "pt-6",
                        p { class: "text-3xl font-bold text-purple-600", "—" }
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
