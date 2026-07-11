use dioxus::prelude::*;

mod components;
use components::auth::{SignIn, SignUp};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_signup = use_signal(|| false);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        
        div { class: "min-h-screen bg-gray-50 py-12 flex flex-col sm:px-6 lg:px-8 text-black",
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
                        SignIn {}
                    }
                }
            }
        }
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}




