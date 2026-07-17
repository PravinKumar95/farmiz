use dioxus::prelude::*;

#[component]
pub fn Feed() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold tracking-tight", "Feed Mill Data" }
            p { class: "text-muted-foreground", "Placeholder for feed batches and production." }
        }
    }
}
