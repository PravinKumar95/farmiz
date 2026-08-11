use dioxus::prelude::*;

use crate::layouts::PublicLayout;
use crate::screens::{login::SignIn, login::SignUp};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum PublicRoute {
    #[layout(PublicLayout)]
    #[route("/")]
    #[redirect("/signin", || PublicRoute::SignIn {})]
    #[redirect("/dashboard", || PublicRoute::SignIn {})]
    #[redirect("/production", || PublicRoute::SignIn {})]
    #[redirect("/sales", || PublicRoute::SignIn {})]
    #[redirect("/purchases", || PublicRoute::SignIn {})]
    #[redirect("/feed", || PublicRoute::SignIn {})]
    #[redirect("/labor", || PublicRoute::SignIn {})]
    #[redirect("/parties", || PublicRoute::SignIn {})]
    #[redirect("/employees", || PublicRoute::SignIn {})]
    #[redirect("/settings", || PublicRoute::SignIn {})]
    SignIn {},
    #[route("/signup")]
    SignUp,
    #[route("/:.._route")]
    PageNotFound { _route: Vec<String> },
}

#[component]
fn PageNotFound(_route: Vec<String>) -> Element {
    let nav = dioxus_router::hooks::use_navigator();
    use_effect(move || {
        nav.replace(PublicRoute::SignIn {});
    });

    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-[50vh] text-center",
            h1 { class: "text-4xl font-bold text-gray-800 dark:text-gray-200", "404" }
            p { class: "text-lg text-gray-600 dark:text-gray-400 mt-2", "Page not found" }
            crate::components::button::Button {
                class: "mt-6",
                onclick: move |_| {
                    nav.push(PublicRoute::SignIn {});
                },
                "Go to Sign In"
            }
        }
    }
}
