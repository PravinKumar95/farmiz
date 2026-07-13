use dioxus::prelude::*;

mod components;
use components::auth::{SignIn, SignUp};
use components::dashboard::Dashboard;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

use dioxus_sdk::storage::use_persistent;

#[component]
fn App() -> Element {
    let mut auth_token = use_persistent("auth_token", || None::<String>);
    let mut auth_email = use_persistent("auth_email", || None::<String>);
    let mut show_signup = use_signal(|| false);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "min-h-screen bg-gray-50 py-12 flex flex-col sm:px-6 lg:px-8 text-black",
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
                div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                    div { class: "bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10 border border-gray-200",
                        div { class: "flex justify-center gap-6 mb-8 text-lg",
                            button {
                                class: if !show_signup() { "font-bold text-blue-600 border-b-2 border-blue-600 pb-1" } else { "text-gray-500 hover:text-gray-700 pb-1" },
                                onclick: move |_| show_signup.set(false),
                                "Sign In"
                            }
                            button {
                                class: if show_signup() { "font-bold text-blue-600 border-b-2 border-blue-600 pb-1" } else { "text-gray-500 hover:text-gray-700 pb-1" },
                                onclick: move |_| show_signup.set(true),
                                "Sign Up"
                            }
                        }
                        if show_signup() {
                            SignUp {}
                        } else {
                            SignIn {
                                on_login: move |info: components::auth::LoginInfo| {
                                    auth_token.set(Some(info.token));
                                    auth_email.set(Some(info.email));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
