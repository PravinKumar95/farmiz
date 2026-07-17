use dioxus::prelude::*;

#[component]
pub fn Purchases() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold tracking-tight", "Purchases & Inventory" }
            p { class: "text-muted-foreground", "Placeholder for material purchases (maize, soya, etc.)." }
        }
    }
}
