use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ViewLayout {
    #[default]
    Table,
    Cards,
}

#[derive(Props, Clone, PartialEq)]
pub struct LayoutToggleProps {
    pub selected: ViewLayout,
    pub onchange: EventHandler<ViewLayout>,
}

#[component]
pub fn LayoutToggle(props: LayoutToggleProps) -> Element {
    let is_table = props.selected == ViewLayout::Table;
    rsx! {
        div { class: "inline-flex p-0.5 bg-stone-100 dark:bg-stone-800/90 rounded-lg border border-stone-200/80 dark:border-stone-700/60 text-xs font-medium shrink-0",
            button {
                r#type: "button",
                class: if is_table {
                    "px-2.5 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all flex items-center gap-1.5"
                } else {
                    "px-2.5 py-1 rounded-md text-stone-500 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1.5"
                },
                onclick: move |_| props.onchange.call(ViewLayout::Table),
                "📊 Table"
            }
            button {
                r#type: "button",
                class: if !is_table {
                    "px-2.5 py-1 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all flex items-center gap-1.5"
                } else {
                    "px-2.5 py-1 rounded-md text-stone-500 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1.5"
                },
                onclick: move |_| props.onchange.call(ViewLayout::Cards),
                "🗂️ Cards"
            }
        }
    }
}
