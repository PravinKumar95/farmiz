use dioxus::prelude::*;

#[component]
pub fn Sales() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold tracking-tight", "Sales Management" }
            p { class: "text-muted-foreground", "Placeholder for standard and broken egg sales." }
        }
    }
}
