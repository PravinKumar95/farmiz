use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use crate::services::{use_broken_egg_sales, use_egg_sales};

#[component]
pub fn Sales() -> Element {
    let mut active_tab = use_signal(|| Some("standard".to_string()));
    let standard_sales = use_egg_sales();
    let broken_sales = use_broken_egg_sales();

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Sales" }
                Button { "Add Sale" }
            }

            Tabs {
                value: active_tab,
                on_value_change: move |v: String| active_tab.set(Some(v)),
                TabList { class: "grid w-full grid-cols-2",
                    TabTrigger { value: "standard", index: 0usize, "Standard Eggs" }
                    TabTrigger { value: "broken", index: 1usize, "Broken Eggs" }
                }

                TabContent { value: "standard", index: 0usize,
                    div { class: "flex flex-col gap-3 mt-4",
                        for sale in standard_sales() {
                            Card { key: "{sale.id}",
                                CardHeader {
                                    div { class: "flex justify-between items-center text-sm",
                                        span { class: "font-bold text-base", "{sale.party_name}" }
                                        span { class: "text-muted-foreground", "{sale.date}" }
                                    }
                                }
                                CardContent {
                                    div { class: "flex justify-between",
                                        div {
                                            div { class: "text-xs text-muted-foreground", "Boxes / Trays" }
                                            div { class: "font-medium", "{sale.quantity_boxes} ({sale.size})" }
                                        }
                                        div {
                                            div { class: "text-xs text-muted-foreground", "Net Rate" }
                                            div { class: "font-medium", "₹{sale.net_rate}" }
                                        }
                                        div {
                                            div { class: "text-xs text-muted-foreground", "Total Eggs" }
                                            div { class: "font-medium", "{sale.total_eggs}" }
                                        }
                                    }
                                }
                                CardFooter {
                                    div { class: "flex justify-between items-center w-full",
                                        div { class: "font-bold text-lg", "₹{sale.total_amount}" }
                                        div { class: "flex flex-col items-end",
                                            div { class: "text-xs text-muted-foreground", "Paid: ₹{sale.received_amount} ({sale.payment_mode})" }
                                            div {
                                                class: if sale.balance < 0.0 { "text-red-500 font-semibold text-sm" } else { "text-green-500 font-semibold text-sm" },
                                                "Bal: ₹{sale.balance}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                TabContent { value: "broken", index: 1usize,
                    div { class: "flex flex-col gap-3 mt-4",
                        for sale in broken_sales() {
                            Card { key: "{sale.id}",
                                CardHeader {
                                    div { class: "flex justify-between items-center text-sm",
                                        span { class: "font-bold text-base", "{sale.bakery_name}" }
                                        span { class: "text-muted-foreground", "{sale.date}" }
                                    }
                                }
                                CardContent {
                                    div { class: "flex justify-between",
                                        div {
                                            div { class: "text-xs text-muted-foreground", "Trays Sold" }
                                            div { class: "font-medium", "{sale.trays_sold}" }
                                        }
                                        div {
                                            div { class: "text-xs text-muted-foreground", "Rate/Tray" }
                                            div { class: "font-medium", "₹{sale.rate}" }
                                        }
                                        div { class: "text-right",
                                            div { class: "text-xs text-muted-foreground", "Returned Trays" }
                                            div { class: "font-medium text-orange-500", "{sale.return_trays}" }
                                        }
                                    }
                                }
                                CardFooter {
                                    div { class: "flex justify-between items-center w-full",
                                        div { class: "font-bold text-lg", "₹{sale.amount}" }
                                        div { class: "flex flex-col items-end",
                                            div { class: "text-xs text-muted-foreground", "Paid: ₹{sale.payment_received}" }
                                            div {
                                                class: if sale.balance_amount > 0.0 { "text-green-500 font-semibold text-sm" } else { "text-red-500 font-semibold text-sm" },
                                                "Bal: ₹{sale.balance_amount}"
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
    }
}
