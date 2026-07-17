use dioxus::prelude::*;

use crate::layouts::AuthenticatedLayout;
use crate::screens::dashboard::Dashboard;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum AuthenticatedRoute {
    #[layout(AuthenticatedLayout)]
    #[route("/")]
    #[redirect("/signin", || AuthenticatedRoute::Dashboard {})]
    #[redirect("/signup", || AuthenticatedRoute::Dashboard {})]
    Dashboard {},
    #[route("/:.._route")]
    PageNotFound { _route: Vec<String> },
}

#[component]
fn PageNotFound(_route: Vec<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-[50vh] text-center",
            h1 { class: "text-4xl font-bold text-gray-800 dark:text-gray-200", "404" }
            p { class: "text-lg text-gray-600 dark:text-gray-400 mt-2", "Page not found" }
            crate::components::button::Button {
                class: "mt-6",
                onclick: move |_| {
                    let nav = dioxus_router::hooks::use_navigator();
                    nav.push(AuthenticatedRoute::Dashboard {});
                },
                "Go to Dashboard"
            }
        }
    }
}
