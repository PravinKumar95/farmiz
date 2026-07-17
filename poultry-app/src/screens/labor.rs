use dioxus::prelude::*;

use crate::components::button::Button;
use crate::components::card::{Card, CardContent, CardHeader};
use crate::services::use_labor_records;

#[component]
pub fn Labor() -> Element {
    let records = use_labor_records();

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Labor" }
                Button { "Log Advance/Attn" }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for record in records() {
                    Card { key: "{record.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center text-sm",
                                span { class: "font-bold text-base", "{record.employee_name}" }
                                span { class: "text-muted-foreground", "{record.date}" }
                            }
                        }
                        CardContent {
                            div { class: "flex justify-between items-center",
                                div {
                                    div { class: "text-xs text-muted-foreground", "Attendance" }
                                    div {
                                        class: if record.attendance == 1.0 {
                                            "font-medium text-green-500"
                                        } else if record.attendance == 0.5 {
                                            "font-medium text-yellow-500"
                                        } else {
                                            "font-medium text-red-500"
                                        },
                                        "{record.attendance} Day"
                                    }
                                }
                                div { class: "text-right",
                                    div { class: "text-xs text-muted-foreground", "Advance Given" }
                                    div { class: "font-bold text-lg", "₹{record.advance_given}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
