use dioxus::prelude::*;

mod components;
use components::auth::{SignIn, SignUp};
use components::dashboard::Dashboard;
use components::tabs::{Tabs, TabList, TabTrigger, TabContent};
use components::card::{Card, CardContent};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// ── Main Application Entry Point ───────────────────────────────────────────────
fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        dioxus_sdk::storage::set_dir!();
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut auth_token = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>("auth_token".to_string(), || None::<String>);
    let mut auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>("auth_email".to_string(), || None::<String>);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "min-h-[100dvh] py-12 flex flex-col items-center px-4 sm:px-6 lg:px-8",
            if auth_token().is_some() {
                // ── Authenticated: show Dashboard ──
                Dashboard {
                    user_email: auth_email().unwrap_or_default(),
                    on_signout: move |_| {
                        auth_token.set(None);
                        auth_email.set(None);
                    }
                }
            } else {
                // ── Not authenticated: show Sign In / Sign Up ──
                div { class: "w-full max-w-md",
                    Tabs {
                        default_value: "signin".to_string(),
                        class: "w-full",
                        TabList { class: "!grid !w-full !grid-cols-2",
                            TabTrigger { value: "signin".to_string(), index: 0usize, "Sign In" }
                            TabTrigger { value: "signup".to_string(), index: 1usize, "Sign Up" }
                        }
                        
                        Card {
                            class: "mt-6 shadow-md",
                            CardContent {
                                class: "pt-6",
                                TabContent { value: "signin".to_string(), index: 0usize,
                                    SignIn {
                                        on_login: move |info: components::auth::LoginInfo| {
                                            auth_token.set(Some(info.token));
                                            auth_email.set(Some(info.email));
                                        }
                                    }
                                }
                                TabContent { value: "signup".to_string(), index: 1usize,
                                    SignUp {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
