use dioxus::prelude::*;
use crate::components::button::{Button, ButtonVariant};

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmDialogProps {
    pub is_open: bool,
    pub title: String,
    pub description: String,
    pub onconfirm: EventHandler<()>,
    pub oncancel: EventHandler<()>,
}

#[component]
pub fn ConfirmDialog(props: ConfirmDialogProps) -> Element {
    if !props.is_open {
        return rsx! {};
    }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4",
            div { class: "w-full max-w-md rounded-xl bg-white dark:bg-stone-900 border border-gray-200 dark:border-stone-800 p-6 shadow-xl flex flex-col gap-4",
                h3 { class: "text-lg font-bold text-gray-900 dark:text-gray-100", "{props.title}" }
                p { class: "text-sm text-gray-600 dark:text-gray-400", "{props.description}" }
                div { class: "flex items-center justify-end gap-3 mt-4",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| props.oncancel.call(()),
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| props.onconfirm.call(()),
                        "Delete"
                    }
                }
            }
        }
    }
}
