use dioxus::prelude::*;

#[component]
pub fn Labor() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold tracking-tight", "Labor Management" }
            p { class: "text-muted-foreground", "Placeholder for employee attendance and advances." }
        }
    }
}
