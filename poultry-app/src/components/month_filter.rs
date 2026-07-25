use dioxus::prelude::*;
use chrono::{Datelike, Utc};

#[derive(Props, Clone, PartialEq)]
pub struct MonthFilterProps {
    pub selected: Option<String>, // e.g. Some("2026-07") or None for "ALL"
    pub onchange: EventHandler<Option<String>>,
}

struct MonthItem {
    key: String,       // "2026-07" or "ALL"
    label: String,     // "Jul 2026" or "All Time"
    opt: Option<String>,
}

#[component]
pub fn MonthFilter(props: MonthFilterProps) -> Element {
    let now = Utc::now();
    let current_year = now.year();
    let current_month = now.month();

    let mut month_options: Vec<MonthItem> = Vec::new();
    month_options.push(MonthItem {
        key: "ALL".to_string(),
        label: "All Time".to_string(),
        opt: None,
    });

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut y = current_year;
    let mut m = current_month as i32;

    for _ in 0..12 {
        let key_str = format!("{:04}-{:02}", y, m);
        let label_str = format!("{} {}", month_names[(m - 1) as usize], y);
        month_options.push(MonthItem {
            key: key_str.clone(),
            label: label_str,
            opt: Some(key_str),
        });

        m -= 1;
        if m < 1 {
            m = 12;
            y -= 1;
        }
    }

    rsx! {
        div { class: "flex items-center gap-2.5 overflow-x-auto flex-nowrap py-2 px-1 w-full max-w-full border-b border-stone-200/60 dark:border-stone-800 mb-3 touch-pan-x",
            style: "-webkit-overflow-scrolling: touch; scrollbar-width: none;",
            for item in month_options {
                {
                    let is_active = match (&props.selected, &item.opt) {
                        (None, None) => true,
                        (Some(s), Some(o)) => s == o,
                        _ => false,
                    };
                    let opt_val = item.opt.clone();
                    rsx! {
                        button {
                            key: "{item.key}",
                            r#type: "button",
                            class: if is_active {
                                "px-4 py-2 rounded-full bg-blue-600 dark:bg-blue-600 text-white text-xs font-bold shadow-md whitespace-nowrap shrink-0 transition-all cursor-pointer ring-2 ring-blue-400/30"
                            } else {
                                "px-4 py-2 rounded-full bg-stone-200/80 dark:bg-stone-800 dark:border dark:border-stone-700/70 text-stone-700 dark:text-stone-300 hover:bg-stone-300 dark:hover:bg-stone-700 text-xs font-medium whitespace-nowrap shrink-0 transition-all cursor-pointer"
                            },
                            onclick: move |_| props.onchange.call(opt_val.clone()),
                            "{item.label}"
                        }
                    }
                }
            }
        }
    }
}
