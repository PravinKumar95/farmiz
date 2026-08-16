use dioxus::prelude::*;

use crate::layouts::AuthenticatedLayout;
use crate::screens::dashboard::Dashboard;
use crate::screens::feed::Feed;
use crate::screens::labor::Labor;
use crate::screens::parties::Parties;
use crate::screens::party_detail::PartyDetail;
use crate::screens::production::Production;
use crate::screens::purchases::Purchases;
use crate::screens::sales::Sales;
use crate::screens::settings::Settings;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum AuthenticatedRoute {
    #[layout(AuthenticatedLayout)]
    #[route("/")]
    #[redirect("/signin", || AuthenticatedRoute::Dashboard {})]
    #[redirect("/signup", || AuthenticatedRoute::Dashboard {})]
    Dashboard {},
    #[route("/production")]
    Production {},
    #[route("/sales")]
    Sales {},
    #[route("/purchases")]
    Purchases {},
    #[route("/feed")]
    Feed {},
    #[route("/labor")]
    #[redirect("/employees", || AuthenticatedRoute::Labor {})]
    Labor {},
    #[route("/parties")]
    Parties {},
    #[route("/settings")]
    Settings {},
    #[route("/parties/:id")]
    PartyDetail { id: String },
    #[route("/:.._route")]
    PageNotFound { _route: Vec<String> },
}

#[component]
fn PageNotFound(_route: Vec<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-[50vh] text-center",
            h1 { class: "text-4xl font-bold text-gray-800 dark:text-gray-200", "404" }
            p { class: "text-lg text-gray-600 dark:text-gray-400 dark:text-gray-400 mt-2", "Page not found" }
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
