use dioxus::prelude::*;

#[component]
pub fn Dashboard(user_email: String, on_signout: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-6 p-6 max-w-2xl mx-auto",

            // Header
            div {
                class: "flex items-center justify-between",
                h1 { class: "text-3xl font-bold", "🐔 Farmiz Dashboard" }
                button {
                    class: "text-sm text-gray-500 hover:text-red-600 transition-colors border border-gray-300 hover:border-red-300 px-3 py-1.5 rounded-lg",
                    onclick: move |_| on_signout.call(()),
                    "Sign Out"
                }
            }

            // Welcome card
            div {
                class: "bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-xl p-5",
                p { class: "text-gray-600 text-sm", "Welcome back," }
                p { class: "text-lg font-semibold text-gray-900 mt-1", "{user_email}" }
            }

            // Quick stats placeholder
            div {
                class: "grid grid-cols-2 gap-4",
                div {
                    class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",
                    p { class: "text-3xl font-bold text-green-600", "—" }
                    p { class: "text-sm text-gray-500 mt-1", "Active Flocks" }
                }
                div {
                    class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",
                    p { class: "text-3xl font-bold text-amber-600", "—" }
                    p { class: "text-sm text-gray-500 mt-1", "Total Birds" }
                }
                div {
                    class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",
                    p { class: "text-3xl font-bold text-blue-600", "—" }
                    p { class: "text-sm text-gray-500 mt-1", "Eggs Today" }
                }
                div {
                    class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",
                    p { class: "text-3xl font-bold text-purple-600", "—" }
                    p { class: "text-sm text-gray-500 mt-1", "Feed Stock (kg)" }
                }
            }

            // Getting started
            div {
                class: "bg-white border border-gray-200 rounded-xl p-5 shadow-sm",
                h2 { class: "text-lg font-semibold mb-3", "🚀 Getting Started" }
                p { class: "text-gray-600 text-sm",
                    "Your account is set up and ready to go. Dashboard features are coming soon!"
                }
            }
        }
    }
}
