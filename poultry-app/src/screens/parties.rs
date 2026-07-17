use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent};
use crate::services::use_parties;

#[component]
pub fn Parties() -> Element {
    let parties = use_parties();

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Parties Directory" }
                Button { "Add Party" }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for party in parties.cloned().unwrap_or_default() {
                    Card { key: "{party.id}",
                        CardContent {
                            div { class: "flex justify-between items-center pt-4",
                                div { class: "flex flex-col",
                                    span { class: "font-bold text-base", "{party.name}" }
                                    span { class: "text-xs text-muted-foreground mt-1 uppercase tracking-wider", "{party.party_type}" }
                                }
                                div { class: "flex flex-col items-end",
                                    div { class: "text-xs text-muted-foreground mb-1", "Current Balance" }
                                    div {
                                        class: if party.current_balance < 0.0 {
                                            "font-bold text-lg text-red-500"
                                        } else {
                                            "font-bold text-lg text-green-500"
                                        },
                                        "₹{party.current_balance.abs()}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
