use dioxus::prelude::*;

#[component]
pub fn Parties() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold tracking-tight", "Parties Directory" }
            p { class: "text-muted-foreground", "Placeholder for the unified ledger of all parties." }
        }
    }
}
