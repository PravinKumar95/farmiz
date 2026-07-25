use dioxus::prelude::*;
use chrono::Utc;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::employee_select::EmployeeSelect;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::month_filter::MonthFilter;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::LaborRecord;
use crate::services::*;

#[component]
pub fn Labor() -> Element {
    let mut records = use_labor_records();
    let api = use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let current_month_str = Utc::now().format("%Y-%m").to_string();
    let mut selected_month = use_signal(move || Some(current_month_str.clone()));
    let mut search_query = use_signal(String::new);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_employee_name = use_signal(String::new);
    let mut form_employee_id = use_signal(|| Option::<String>::None);
    let mut form_attendance = use_signal(|| "1.0".to_string());
    let mut form_advance = use_signal(|| "0.0".to_string());
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let employee = form_employee_name().trim().to_string();

        if date.is_empty() {
            form_error.set("Date required".to_string());
            return;
        }
        if employee.is_empty() {
            form_error.set("Employee selection required".to_string());
            return;
        }

        let attendance: f64 = match form_attendance().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid attendance".to_string());
                return;
            }
        };
        let advance: f64 = match form_advance().parse() {
            Ok(v) => v,
            Err(_) => {
                form_error.set("Invalid advance amount".to_string());
                return;
            }
        };

        let new_record = LaborRecord {
            id: form_id().unwrap_or_default(),
            date,
            employee_name: employee,
            employee_id: form_employee_id(),
            attendance,
            advance_given: advance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/labor/{}", id), &new_record).await
            } else {
                api.post("/api/labor", &new_record).await
            };

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    records.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    let confirm_delete = move |_| {
        if let Some(id) = delete_id() {
            spawn(async move {
                if let Ok(_) = api.delete(&format!("/api/labor/{}", id)).await {
                    delete_id.set(None);
                    records.restart();
                }
            });
        }
    };

    let rec_list = records.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let q = search_query().trim().to_lowercase();
    let filtered_records: Vec<_> = rec_list.into_iter().filter(|r| {
        let matches_month = if let Some(ref m) = selected_month() {
            r.date.starts_with(m)
        } else {
            true
        };
        let matches_search = if q.is_empty() {
            true
        } else {
            r.employee_name.to_lowercase().contains(&q) || r.date.contains(&q)
        };
        matches_month && matches_search
    }).collect();

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-3",
                div { class: "flex justify-between items-center",
                    h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Labor & Attendance Log" }
                    Button {
                        onclick: move |_| {
                            form_id.set(None);
                            form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                            form_employee_name.set(String::new());
                            form_employee_id.set(None);
                            form_attendance.set("1.0".to_string());
                            form_advance.set("0.0".to_string());
                            form_error.set(String::new());
                            is_sheet_open.set(true);
                        },
                        "Add Record"
                    }
                }

                Input {
                    placeholder: "🔍 Search by employee name or date...",
                    value: "{search_query}",
                    oninput: move |e: Event<FormData>| search_query.set(e.value())
                }
                MonthFilter {
                    selected: selected_month(),
                    onchange: move |m| selected_month.set(m)
                }
            }

            // SECTION 2: SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
                    match records.cloned() {
                        Some(Ok(_)) if !filtered_records.is_empty() => rsx! {
                            div { class: "flex flex-col gap-3 mt-2",
                                for item in filtered_records {
                                    Card { key: "{item.id}",
                                        CardHeader {
                                            div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                span { "{item.date}" }
                                                span { "Attendance: {item.attendance} Day(s)" }
                                            }
                                        }
                                        CardContent {
                                            div { class: "flex flex-col gap-3",
                                                div { class: "flex justify-between items-baseline",
                                                    span { class: "font-bold text-xl text-gray-900 dark:text-gray-100", "{item.employee_name}" }
                                                    span { class: "text-base font-bold text-amber-600 dark:text-amber-400", "Advance: ₹{item.advance_given:.2}" }
                                                }
                                            }
                                        }
                                        CardFooter {
                                            div { class: "flex justify-end gap-2 w-full pt-1",
                                                {
                                                    let edit_item = item.clone();
                                                    let del_id = item.id.clone();
                                                    rsx! {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                form_id.set(Some(edit_item.id.clone()));
                                                                form_date.set(edit_item.date.clone());
                                                                form_employee_name.set(edit_item.employee_name.clone());
                                                                form_employee_id.set(edit_item.employee_id.clone());
                                                                form_attendance.set(edit_item.attendance.to_string());
                                                                form_advance.set(edit_item.advance_given.to_string());
                                                                is_sheet_open.set(true);
                                                            },
                                                            "Edit"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                delete_id.set(Some(del_id.clone()));
                                                            },
                                                            "Delete"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Ok(_)) => rsx! {
                            EmptyState {
                                icon: "👥",
                                title: "No Labor Records Found",
                                description: "No labor/attendance records match your search criteria or date filter."
                            }
                        },
                        Some(Err(err)) => rsx! { ErrorState { message: err } },
                        None => rsx! { LoadingState {} }
                    }

                    ConfirmDialog {
                        is_open: delete_id().is_some(),
                        title: "Delete Labor Record".to_string(),
                        description: "Are you sure you want to delete this labor record? Employee balance will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| delete_id.set(None)
                    }
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if form_id().is_some() { "Edit Labor Record" } else { "Log Labor & Advance" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "labor-date", "Date" }
                            Input {
                                r#type: "date",
                                value: "{form_date}",
                                oninput: move |e: Event<FormData>| form_date.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "labor-emp", "Employee" }
                            EmployeeSelect {
                                value: form_employee_id().unwrap_or_else(|| form_employee_name()),
                                onchange: move |(eid, ename): (String, String)| {
                                    if !eid.is_empty() {
                                        form_employee_id.set(Some(eid));
                                    } else {
                                        form_employee_id.set(None);
                                    }
                                    form_employee_name.set(ename);
                                }
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "labor-att", "Attendance (Full/Half Day)" }
                                Input {
                                    r#type: "number",
                                    step: "0.5",
                                    value: "{form_attendance}",
                                    oninput: move |e: Event<FormData>| form_attendance.set(e.value())
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "labor-adv", "Advance Given (₹)" }
                                Input {
                                    r#type: "number",
                                    step: "1",
                                    value: "{form_advance}",
                                    oninput: move |e: Event<FormData>| form_advance.set(e.value())
                                }
                            }
                        }
                    }
                    SheetFooter {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| is_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            onclick: submit_handler,
                            if form_id().is_some() { "Update Record" } else { "Save Record" }
                        }
                    }
                }
            }
        }
    }
}
