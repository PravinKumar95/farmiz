use dioxus::prelude::*;
use chrono::{Datelike, Duration, NaiveDate, Utc};

#[derive(Clone, PartialEq, Debug)]
pub struct DateRange {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub preset: String, // "ALL", "TODAY", "LAST_7_DAYS", "THIS_MONTH", "LAST_MONTH", "CUSTOM"
}

impl Default for DateRange {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            preset: "ALL".to_string(),
        }
    }
}

impl DateRange {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn is_all(&self) -> bool {
        self.start_date.is_none() && self.end_date.is_none()
    }

    pub fn matches(&self, date_str: &str) -> bool {
        if date_str.is_empty() {
            return true;
        }

        let d = if date_str.len() >= 10 {
            &date_str[..10]
        } else {
            date_str
        };

        if let Some(ref start) = self.start_date {
            if !start.is_empty() && d < start.as_str() {
                return false;
            }
        }
        if let Some(ref end) = self.end_date {
            if !end.is_empty() && d > end.as_str() {
                return false;
            }
        }
        true
    }
}

fn get_preset_dates(preset: &str) -> (Option<String>, Option<String>) {
    let now = Utc::now().date_naive();
    match preset {
        "TODAY" => {
            let today_str = now.format("%Y-%m-%d").to_string();
            (Some(today_str.clone()), Some(today_str))
        }
        "LAST_7_DAYS" => {
            let start = now - Duration::days(6);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(now.format("%Y-%m-%d").to_string()),
            )
        }
        "THIS_MONTH" => {
            let start = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or(now);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(now.format("%Y-%m-%d").to_string()),
            )
        }
        "LAST_MONTH" => {
            let (prev_year, prev_month) = if now.month() == 1 {
                (now.year() - 1, 12)
            } else {
                (now.year(), now.month() - 1)
            };
            let start = NaiveDate::from_ymd_opt(prev_year, prev_month, 1).unwrap_or(now);
            let next_start = if prev_month == 12 {
                NaiveDate::from_ymd_opt(prev_year + 1, 1, 1).unwrap_or(now)
            } else {
                NaiveDate::from_ymd_opt(prev_year, prev_month + 1, 1).unwrap_or(now)
            };
            let end = next_start - Duration::days(1);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
            )
        }
        _ => (None, None),
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DateRangeFilterProps {
    pub selected: DateRange,
    pub onchange: EventHandler<DateRange>,
}

#[component]
pub fn DateRangeFilter(props: DateRangeFilterProps) -> Element {
    let presets = [
        ("ALL", "All Time"),
        ("TODAY", "Today"),
        ("LAST_7_DAYS", "Last 7 Days"),
        ("THIS_MONTH", "This Month"),
        ("LAST_MONTH", "Last Month"),
        ("CUSTOM", "📅 Custom"),
    ];

    let current_preset = props.selected.preset.clone();
    let current_start = props.selected.start_date.clone().unwrap_or_default();
    let current_end = props.selected.end_date.clone().unwrap_or_default();

    let is_custom = current_preset == "CUSTOM";
    let has_filter = props.selected.start_date.is_some() || props.selected.end_date.is_some();

    rsx! {
        div { class: "flex flex-col gap-1.5 w-full py-0.5",
            div { class: "flex items-center justify-between gap-2 w-full",
                // ── Preset Pills (Horizontal Scroll) ──
                div { class: "flex items-center gap-1 overflow-x-auto flex-nowrap py-0.5 max-w-full touch-pan-x",
                    style: "-webkit-overflow-scrolling: touch; scrollbar-width: none;",
                    for (key, label) in presets {
                        {
                            let is_active = current_preset == key;
                            let k_str = key.to_string();
                            let s_opt = props.selected.start_date.clone();
                            let e_opt = props.selected.end_date.clone();
                            rsx! {
                                button {
                                    key: "{key}",
                                    r#type: "button",
                                    class: if is_active {
                                        "px-2.5 py-1 rounded-md bg-blue-600 dark:bg-blue-600 text-white text-[11px] font-semibold shadow-xs whitespace-nowrap shrink-0 transition-all cursor-pointer ring-1 ring-blue-500/50"
                                    } else {
                                        "px-2.5 py-1 rounded-md bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 hover:bg-stone-200/80 dark:hover:bg-stone-700/80 text-[11px] font-medium whitespace-nowrap shrink-0 transition-all cursor-pointer border border-stone-200/70 dark:border-stone-700/60"
                                    },
                                    onclick: {
                                        let k = k_str.clone();
                                        let s_val = s_opt.clone();
                                        let e_val = e_opt.clone();
                                        move |_| {
                                            if k == "CUSTOM" {
                                                props.onchange.call(DateRange {
                                                    start_date: s_val.clone(),
                                                    end_date: e_val.clone(),
                                                    preset: "CUSTOM".to_string(),
                                                });
                                            } else {
                                                let (s, e) = get_preset_dates(&k);
                                                props.onchange.call(DateRange {
                                                    start_date: s,
                                                    end_date: e,
                                                    preset: k.clone(),
                                                });
                                            }
                                        }
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }

                if has_filter && !is_custom {
                    button {
                        r#type: "button",
                        class: "px-2 py-0.5 rounded bg-stone-200 dark:bg-stone-700 hover:bg-stone-300 dark:hover:bg-stone-600 text-stone-700 dark:text-stone-300 font-medium text-[10px] whitespace-nowrap transition-colors cursor-pointer shrink-0",
                        onclick: move |_| {
                            props.onchange.call(DateRange::default());
                        },
                        "✕ Reset"
                    }
                }
            }

            // ── Custom Date Pickers & Reset (Only expanded if Custom is chosen) ──
            if is_custom {
                div { class: "flex flex-wrap items-center gap-2 self-start bg-stone-50 dark:bg-stone-800/80 p-1.5 rounded-lg border border-stone-200/80 dark:border-stone-700/80 text-xs mt-0.5",
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-stone-500 dark:text-stone-400 font-medium text-[11px]", "From" }
                        input {
                            r#type: "date",
                            class: "h-6 px-1.5 rounded border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 text-[11px] focus:outline-none focus:ring-1 focus:ring-blue-500",
                            value: "{current_start}",
                            onchange: {
                                let end_val = current_end.clone();
                                move |e: Event<FormData>| {
                                    let val = e.value();
                                    let s = if val.is_empty() { None } else { Some(val) };
                                    let e_opt = if end_val.is_empty() { None } else { Some(end_val.clone()) };
                                    props.onchange.call(DateRange {
                                        start_date: s,
                                        end_date: e_opt,
                                        preset: "CUSTOM".to_string(),
                                    });
                                }
                            }
                        }
                    }

                    div { class: "flex items-center gap-1.5",
                        span { class: "text-stone-500 dark:text-stone-400 font-medium text-[11px]", "To" }
                        input {
                            r#type: "date",
                            class: "h-6 px-1.5 rounded border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 text-[11px] focus:outline-none focus:ring-1 focus:ring-blue-500",
                            value: "{current_end}",
                            onchange: {
                                let start_val = current_start.clone();
                                move |e: Event<FormData>| {
                                    let val = e.value();
                                    let e_opt = if val.is_empty() { None } else { Some(val) };
                                    let s = if start_val.is_empty() { None } else { Some(start_val.clone()) };
                                    props.onchange.call(DateRange {
                                        start_date: s,
                                        end_date: e_opt,
                                        preset: "CUSTOM".to_string(),
                                    });
                                }
                            }
                        }
                    }

                    button {
                        r#type: "button",
                        class: "px-2 py-0.5 rounded bg-stone-200 dark:bg-stone-700 hover:bg-stone-300 dark:hover:bg-stone-600 text-stone-700 dark:text-stone-300 font-medium text-[11px] transition-colors cursor-pointer",
                        onclick: move |_| {
                            props.onchange.call(DateRange::default());
                        },
                        "✕ Clear"
                    }
                }
            }
        }
    }
}
