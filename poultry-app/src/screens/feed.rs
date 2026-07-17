use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardHeader, CardFooter};
use crate::services::use_feed_batches;

#[component]
pub fn Feed() -> Element {
    let batches = use_feed_batches();

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Feed Mill" }
                Button { "Log Batch" }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for batch in batches.cloned().unwrap_or_default() {
                    Card { key: "{batch.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center text-sm",
                                span { class: "font-bold text-base", "Batch: {batch.batch_id}" }
                                span { class: "text-muted-foreground", "{batch.date}" }
                            }
                        }
                        CardContent {
                            div { class: "flex justify-between",
                                div {
                                    div { class: "text-xs text-muted-foreground", "Feed Type" }
                                    div { class: "font-medium", "{batch.feed_type}" }
                                }
                                div { class: "text-right",
                                    div { class: "text-xs text-muted-foreground", "Rate" }
                                    div { class: "font-medium", "₹{batch.rate}" }
                                }
                            }
                        }
                        CardFooter {
                            div { class: "flex flex-col w-full gap-2",
                                div { class: "flex justify-between items-center w-full",
                                    div { class: "text-sm text-muted-foreground", "Total Amount" }
                                    div { class: "font-bold text-lg", "₹{batch.total_amount}" }
                                }
                                div { class: "flex justify-between items-center w-full text-xs text-muted-foreground pt-2 border-t dark:border-stone-800",
                                    div { "Op Bal: ₹{batch.opening_balance}" }
                                    div { "Payment: ₹{batch.payment}" }
                                    div { "Cl Bal: ₹{batch.closing_balance}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
