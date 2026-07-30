use dioxus::prelude::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::skeleton::Skeleton;

#[component]
pub fn EmptyState(
    icon: &'static str,
    title: &'static str,
    description: &'static str,
) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center p-8 text-center rounded-xl border border-dashed border-stone-300 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-900/40 my-4 gap-2 w-full",
            div { class: "text-4xl mb-1 opacity-80", "{icon}" }
            div { class: "font-semibold text-base text-gray-900 dark:text-gray-100", "{title}" }
            p { class: "text-xs text-gray-500 dark:text-gray-400 max-w-sm", "{description}" }
        }
    }
}

#[component]
pub fn LoadingState() -> Element {
    rsx! {
        div { class: "flex flex-col gap-3 w-full my-4",
            div { class: "flex items-center justify-center p-6 gap-3",
                div { class: "inline-block w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" }
                p { class: "text-xs font-medium text-gray-500 dark:text-gray-400", "Loading data..." }
            }
            div { class: "flex flex-col gap-3 w-full",
                Skeleton { class: "h-24 w-full rounded-xl bg-stone-200/70 dark:bg-stone-800/70 animate-pulse" }
                Skeleton { class: "h-24 w-full rounded-xl bg-stone-200/70 dark:bg-stone-800/70 animate-pulse" }
            }
        }
    }
}

#[component]
pub fn ErrorState(
    message: String,
    on_retry: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center p-8 text-center rounded-xl border border-red-200 dark:border-red-900/40 bg-red-50/40 dark:bg-red-950/20 my-4 gap-2 w-full",
            div { class: "text-3xl mb-1 text-red-500", "⚠️" }
            div { class: "font-semibold text-base text-red-600 dark:text-red-400", "Failed to load data" }
            p { class: "text-xs text-gray-600 dark:text-gray-400 max-w-sm", "{message}" }
            if let Some(retry) = on_retry {
                div { class: "mt-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| retry.call(()),
                        "Retry"
                    }
                }
            }
        }
    }
}
