use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::services::use_material_purchases;

#[component]
pub fn Purchases() -> Element {
    let purchases = use_material_purchases();

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Purchases" }
                Button { "Add Purchase" }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for purchase in purchases.cloned().unwrap_or_default() {
                    Card { key: "{purchase.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center text-sm",
                                span { class: "font-bold text-base", "{purchase.party_name}" }
                                span { class: "text-muted-foreground", "{purchase.date}" }
                            }
                        }
                        CardContent {
                            div { class: "flex justify-between",
                                div {
                                    div { class: "text-xs text-muted-foreground", "Material" }
                                    div { class: "font-medium", "{purchase.material_name}" }
                                }
                                div {
                                    div { class: "text-xs text-muted-foreground", "Quantity" }
                                    div { class: "font-medium", "{purchase.quantity_kg} kg" }
                                }
                                div { class: "text-right",
                                    div { class: "text-xs text-muted-foreground", "Rate/kg" }
                                    div { class: "font-medium", "₹{purchase.rate_per_kg}" }
                                }
                            }
                        }
                        CardFooter {
                            div { class: "flex justify-between items-center w-full",
                                div { class: "font-bold text-lg", "₹{purchase.total_amount}" }
                                div { class: "flex flex-col items-end",
                                    div {
                                        class: if purchase.status == "PAID" {
                                            "text-xs px-2 py-1 rounded bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300 font-semibold mb-1"
                                        } else {
                                            "text-xs px-2 py-1 rounded bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300 font-semibold mb-1"
                                        },
                                        "{purchase.status}"
                                    }
                                    div { class: "text-xs text-muted-foreground", "Adv: ₹{purchase.advance_paid}" }
                                    div { class: "text-red-500 font-semibold text-sm", "Bal: ₹{purchase.balance}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
