use dioxus::prelude::*;

#[component]
pub fn Logo(#[props(default = "h-8 w-8")] class: &'static str) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 32 32",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",

            // Minimalist Geometric Egg Arc
            path {
                d: "M 16 3 C 10.5 3 6 8.5 6 16 C 6 23.5 10.5 29 16 29 C 21.5 29 26 23.5 26 16 C 26 13.5 25.2 11.2 23.8 9.2",
                stroke: "currentColor",
                stroke_width: "2.5",
                stroke_linecap: "round",
                class: "text-emerald-500 dark:text-emerald-400 transition-colors",
            }

            // Sharp Minimalist Crest
            path {
                d: "M 16 7 L 21 3 L 20 9 L 26 6 L 23 13",
                stroke: "currentColor",
                stroke_width: "2.5",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                class: "text-emerald-500 dark:text-emerald-400 transition-colors",
            }
        }
    }
}
