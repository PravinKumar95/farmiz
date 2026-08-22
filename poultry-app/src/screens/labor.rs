use dioxus::prelude::*;
use chrono::Utc;
use crate::components::toast::{use_toast, ToastOptions};
use crate::i18n::tr;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::employee_select::EmployeeSelect;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::date_range_filter::{DateRange, DateRangeFilter};
use crate::components::layout_toggle::{LayoutToggle, ViewLayout};
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::{Employee, LaborPayrollSummary, LaborRecord};
use crate::services::*;

#[component]
pub fn Labor() -> Element {
    let mut records = use_labor_records();
    let mut employees_res = use_employees();
    let api = use_auth();
    let toast_api = use_toast();

    // Sub-view: "LOGS", "SETTLEMENT", or "WORKERS"
    let mut view_mode = use_signal(|| "LOGS".to_string());
    let mut view_layout = use_signal(ViewLayout::default);

    // Search & Filter
    let mut date_range = use_signal(DateRange::default);
    let mut search_query = use_signal(String::new);

    // --- Labor Record Form State ---
    let mut is_labor_sheet_open = use_signal(|| false);
    let mut labor_delete_id = use_signal(|| Option::<String>::None);
    let mut labor_form_id = use_signal(|| Option::<String>::None);
    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_employee_name = use_signal(String::new);
    let mut form_employee_id = use_signal(|| Option::<String>::None);
    let mut form_attendance = use_signal(|| "1.0".to_string());
    let mut form_advance = use_signal(|| "0.0".to_string());
    let mut labor_form_error = use_signal(String::new);
    let mut is_labor_submitting = use_signal(|| false);
    let mut is_labor_deleting = use_signal(|| false);

    // --- Worker Form State ---
    let mut is_worker_sheet_open = use_signal(|| false);
    let mut worker_delete_id = use_signal(|| Option::<String>::None);
    let mut worker_form_id = use_signal(|| Option::<String>::None);
    let mut worker_form_name = use_signal(String::new);
    let mut worker_form_role = use_signal(String::new);
    let mut worker_form_wage = use_signal(|| "500.0".to_string());
    let mut worker_form_balance = use_signal(|| "0.0".to_string());
    let mut worker_form_error = use_signal(String::new);
    let mut is_worker_submitting = use_signal(|| false);
    let mut is_worker_deleting = use_signal(|| false);

    // --- Submit Labor Record ---
    let submit_labor_handler = move |_| {
        if is_labor_submitting() {
            return;
        }

        let date = form_date().trim().to_string();
        let employee = form_employee_name().trim().to_string();

        if date.is_empty() {
            labor_form_error.set("Date required".to_string());
            return;
        }
        if employee.is_empty() {
            labor_form_error.set("Employee selection required".to_string());
            return;
        }

        let attendance: f64 = match form_attendance().parse() {
            Ok(v) => v,
            Err(_) => {
                labor_form_error.set("Invalid attendance value".to_string());
                return;
            }
        };
        let advance: f64 = match form_advance().parse() {
            Ok(v) => v,
            Err(_) => {
                labor_form_error.set("Invalid advance amount".to_string());
                return;
            }
        };

        labor_form_error.set(String::new());
        is_labor_submitting.set(true);

        let is_edit = labor_form_id().is_some();
        let new_record = LaborRecord {
            id: labor_form_id().unwrap_or_default(),
            date,
            employee_name: employee,
            employee_id: form_employee_id(),
            attendance,
            advance_given: advance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = labor_form_id() {
                api.put(&format!("/api/labor/{}", id), &new_record).await
            } else {
                api.post("/api/labor", &new_record).await
            };

            is_labor_submitting.set(false);

            match res {
                Ok(_) => {
                    is_labor_sheet_open.set(false);
                    labor_form_error.set(String::new());
                    records.restart();
                    employees_res.restart();
                    let desc = if is_edit {
                        "Labor record updated successfully."
                    } else {
                        "Labor record logged successfully."
                    };
                    toast_api.success(tr("success"), ToastOptions::new().description(desc));
                }
                Err(e) => {
                    labor_form_error.set(e.clone());
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
            }
        });
    };

    // --- Delete Labor Record ---
    let confirm_delete_labor = move |_| {
        if is_labor_deleting() {
            return;
        }
        if let Some(id) = labor_delete_id() {
            is_labor_deleting.set(true);
            spawn(async move {
                let res = api.delete(&format!("/api/labor/{}", id)).await;
                if let Ok(_) = res {
                    labor_delete_id.set(None);
                    records.restart();
                    employees_res.restart();
                    toast_api.success(
                        tr("success"),
                        ToastOptions::new().description("Labor record deleted successfully."),
                    );
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_labor_deleting.set(false);
            });
        }
    };

    // --- Submit Worker ---
    let submit_worker_handler = move |_| {
        if is_worker_submitting() {
            return;
        }

        let name = worker_form_name().trim().to_string();
        let role = worker_form_role().trim().to_string();
        let wage_str = worker_form_wage().trim().to_string();
        let balance_str = worker_form_balance().trim().to_string();

        if name.len() < 2 {
            worker_form_error.set("Name must be at least 2 characters long.".to_string());
            return;
        }
        if role.is_empty() {
            worker_form_error.set("Role is required.".to_string());
            return;
        }
        let daily_wage: f64 = match wage_str.parse() {
            Ok(w) => w,
            Err(_) => {
                worker_form_error.set("Daily wage must be a valid number.".to_string());
                return;
            }
        };
        let current_balance: f64 = match balance_str.parse() {
            Ok(b) => b,
            Err(_) => {
                worker_form_error.set("Balance must be a valid number.".to_string());
                return;
            }
        };

        worker_form_error.set(String::new());
        is_worker_submitting.set(true);

        let is_edit = worker_form_id().is_some();
        let new_employee = Employee {
            id: worker_form_id().unwrap_or_default(),
            name,
            role,
            daily_wage,
            current_balance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = worker_form_id() {
                api.put(&format!("/api/employees/{}", id), &new_employee).await
            } else {
                api.post("/api/employees", &new_employee).await
            };

            is_worker_submitting.set(false);

            match res {
                Ok(_) => {
                    is_worker_sheet_open.set(false);
                    worker_form_name.set(String::new());
                    worker_form_role.set(String::new());
                    worker_form_wage.set("500.0".to_string());
                    worker_form_balance.set("0.0".to_string());
                    employees_res.restart();
                    records.restart();
                    let desc = if is_edit {
                        "Worker profile updated successfully."
                    } else {
                        "Worker added successfully."
                    };
                    toast_api.success(tr("success"), ToastOptions::new().description(desc));
                }
                Err(e) => {
                    worker_form_error.set(e.clone());
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
            }
        });
    };

    // --- Delete Worker ---
    let confirm_delete_worker = move |_| {
        if is_worker_deleting() {
            return;
        }
        if let Some(id) = worker_delete_id() {
            is_worker_deleting.set(true);
            spawn(async move {
                let res = api.delete(&format!("/api/employees/{}", id)).await;
                if let Ok(_) = res {
                    worker_delete_id.set(None);
                    employees_res.restart();
                    records.restart();
                    toast_api.success(
                        tr("success"),
                        ToastOptions::new().description("Worker profile deleted successfully."),
                    );
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_worker_deleting.set(false);
            });
        }
    };

    let rec_list = records.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let emp_list = employees_res.cloned().and_then(|r| r.ok()).unwrap_or_default();

    let q = search_query().trim().to_lowercase();

    // Filtered labor records
    let filtered_records: Vec<_> = rec_list
        .into_iter()
        .filter(|r| {
            let matches_date = date_range().matches(&r.date);
            let matches_search = if q.is_empty() {
                true
            } else {
                r.employee_name.to_lowercase().contains(&q) || r.date.contains(&q)
            };
            matches_date && matches_search
        })
        .collect();

    // Filtered workers list
    let filtered_workers: Vec<_> = emp_list
        .iter()
        .filter(|e| {
            if q.is_empty() {
                true
            } else {
                e.name.to_lowercase().contains(&q) || e.role.to_lowercase().contains(&q)
            }
        })
        .cloned()
        .collect();

    // Group & calculate payroll summary per worker for Settlement view
    let mut payroll_summaries: Vec<LaborPayrollSummary> = Vec::new();
    let mut processed_names = std::collections::HashSet::new();

    for r in &filtered_records {
        if processed_names.contains(&r.employee_name) {
            continue;
        }
        processed_names.insert(r.employee_name.clone());

        let worker_records: Vec<&LaborRecord> = filtered_records
            .iter()
            .filter(|rec| rec.employee_name == r.employee_name)
            .collect();

        let total_att: f64 = worker_records.iter().map(|rec| rec.attendance).sum();
        let total_adv: f64 = worker_records.iter().map(|rec| rec.advance_given).sum();

        let daily_wage = emp_list
            .iter()
            .find(|e| e.name.to_lowercase() == r.employee_name.to_lowercase())
            .map(|e| e.daily_wage)
            .unwrap_or(500.0);

        let total_earned = total_att * daily_wage;
        let net_balance = total_earned - total_adv;

        payroll_summaries.push(LaborPayrollSummary {
            employee_id: r.employee_id.clone(),
            employee_name: r.employee_name.clone(),
            daily_wage,
            total_attendance: total_att,
            total_earned,
            total_advance: total_adv,
            opening_balance: 0.0,
            net_balance,
        });
    }

    let total_att_sum: f64 = filtered_records.iter().map(|r| r.attendance).sum();
    let total_adv_sum: f64 = filtered_records.iter().map(|r| r.advance_given).sum();

    let total_settlement_att: f64 = payroll_summaries.iter().map(|s| s.total_attendance).sum();
    let total_settlement_earned: f64 = payroll_summaries.iter().map(|s| s.total_earned).sum();
    let total_settlement_adv: f64 = payroll_summaries.iter().map(|s| s.total_advance).sum();
    let total_settlement_net: f64 = payroll_summaries.iter().map(|s| s.net_balance).sum();

    let title_str = tr("labor");
    let log_btn_str = tr("log-labor");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // FIXED HEADER
            div { class: "shrink-0 p-3 md:p-6 pb-2 md:pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-2 md:gap-3",
                div { class: "flex justify-between items-center gap-2",
                    div { class: "min-w-0",
                        h1 { class: "text-lg md:text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100 truncate", "👥 {title_str}" }
                        p { class: "hidden sm:block text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Track daily attendance, cash advances, payroll settlements & worker directory" }
                    }
                    if view_mode() == "WORKERS" {
                        Button {
                            onclick: move |_| {
                                worker_form_id.set(None);
                                worker_form_name.set(String::new());
                                worker_form_role.set("Farm Worker".to_string());
                                worker_form_wage.set("500.0".to_string());
                                worker_form_balance.set("0.0".to_string());
                                worker_form_error.set(String::new());
                                is_worker_sheet_open.set(true);
                            },
                            "+ Add Worker"
                        }
                    } else {
                        Button {
                            onclick: move |_| {
                                labor_form_id.set(None);
                                form_date.set(Utc::now().format("%Y-%m-%d").to_string());
                                form_employee_name.set(String::new());
                                form_employee_id.set(None);
                                form_attendance.set("1.0".to_string());
                                form_advance.set("0.0".to_string());
                                labor_form_error.set(String::new());
                                is_labor_sheet_open.set(true);
                            },
                            "+ {log_btn_str}"
                        }
                    }
                }

                // Mode Toggle Bar & Filters
                div { class: "flex flex-col gap-2",
                    div { class: "flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2",
                        div { class: "flex p-0.5 sm:p-1 bg-stone-100 dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 text-xs font-medium self-start shrink-0 overflow-x-auto max-w-full",
                            button {
                                class: if view_mode() == "LOGS" {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-xs transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                } else {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                },
                                onclick: move |_| view_mode.set("LOGS".to_string()),
                                span { "📋" }
                                span { class: "hidden sm:inline", " Daily" }
                                span { " Logs" }
                            }
                            button {
                                class: if view_mode() == "SETTLEMENT" {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-xs transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                } else {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                },
                                onclick: move |_| view_mode.set("SETTLEMENT".to_string()),
                                span { "🧮" }
                                span { class: "hidden sm:inline", " Monthly" }
                                span { " Settlement" }
                            }
                            button {
                                class: if view_mode() == "WORKERS" {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-xs transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                } else {
                                    "px-2 sm:px-3 py-1 sm:py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1 text-xs whitespace-nowrap"
                                },
                                onclick: move |_| view_mode.set("WORKERS".to_string()),
                                span { "👷" }
                                span { class: "hidden sm:inline", " Manage" }
                                span { " Workers" }
                            }
                        }

                        div { class: "flex items-center gap-2 flex-1",
                            div { class: "flex-1 min-w-0",
                                Input {
                                    placeholder: if view_mode() == "WORKERS" { "🔍 Search worker..." } else { "🔍 Search employee or date..." },
                                    value: "{search_query}",
                                    oninput: move |e: Event<FormData>| search_query.set(e.value())
                                }
                            }
                            LayoutToggle {
                                selected: view_layout(),
                                onchange: move |vl| view_layout.set(vl)
                            }
                        }
                    }

                    if view_mode() != "WORKERS" {
                        DateRangeFilter {
                            selected: date_range(),
                            onchange: move |dr| date_range.set(dr)
                        }
                    }
                }
            }

            // SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-3 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-16 md:pb-20",

                    if view_mode() == "WORKERS" {
                        // WORKERS DIRECTORY VIEW
                        match employees_res.cloned() {
                            Some(Ok(_)) if !filtered_workers.is_empty() && view_layout() == ViewLayout::Table => rsx! {
                                Card { class: "mt-2",
                                    CardContent { class: "p-0 overflow-hidden rounded-xl",
                                        div { class: "overflow-x-auto",
                                            table { class: "w-full text-sm text-left",
                                                thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                    tr {
                                                        th { class: "px-6 py-3", "Worker Name" }
                                                        th { class: "px-6 py-3", "Role" }
                                                        th { class: "px-6 py-3 text-right", "Daily Wage" }
                                                        th { class: "px-6 py-3 text-right", "Advance Balance" }
                                                        th { class: "px-6 py-3 text-right", "Actions" }
                                                    }
                                                }
                                                tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                    for emp in filtered_workers.iter() {
                                                        tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                            td { class: "px-6 py-4 font-medium text-gray-900 dark:text-gray-100", "{emp.name}" }
                                                            td { class: "px-6 py-4 text-xs font-semibold text-gray-600 dark:text-gray-300", "{emp.role}" }
                                                            td { class: "px-6 py-4 text-right font-semibold text-gray-900 dark:text-gray-100", "₹ {emp.daily_wage:.2}" }
                                                            td { class: "px-6 py-4 text-right font-bold text-amber-600 dark:text-amber-400", "₹ {emp.current_balance:.2}" }
                                                            td { class: "px-6 py-4 text-right flex items-center justify-end gap-2",
                                                                {
                                                                    let e_edit = emp.clone();
                                                                    let e_del_id = emp.id.clone();
                                                                    rsx! {
                                                                        Button {
                                                                            variant: ButtonVariant::Outline,
                                                                            onclick: move |_| {
                                                                                worker_form_id.set(Some(e_edit.id.clone()));
                                                                                worker_form_name.set(e_edit.name.clone());
                                                                                worker_form_role.set(e_edit.role.clone());
                                                                                worker_form_wage.set(e_edit.daily_wage.to_string());
                                                                                worker_form_balance.set(e_edit.current_balance.to_string());
                                                                                is_worker_sheet_open.set(true);
                                                                            },
                                                                            "{edit_str}"
                                                                        }
                                                                        Button {
                                                                            variant: ButtonVariant::Destructive,
                                                                            onclick: move |_| {
                                                                                worker_delete_id.set(Some(e_del_id.clone()));
                                                                            },
                                                                            "{delete_str}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Ok(_)) if !filtered_workers.is_empty() => rsx! {
                                div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2",
                                    for emp in filtered_workers {
                                        Card { key: "{emp.id}",
                                            CardHeader {
                                                div { class: "flex justify-between items-center",
                                                    div {
                                                        h3 { class: "text-lg font-bold text-gray-900 dark:text-gray-100", "{emp.name}" }
                                                        p { class: "text-xs text-gray-500 dark:text-gray-400", "Role: {emp.role}" }
                                                    }
                                                    span { class: "px-2.5 py-1 rounded-full text-xs font-bold bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800",
                                                        "₹{emp.daily_wage:.2}/day"
                                                    }
                                                }
                                            }
                                            CardContent {
                                                div { class: "flex justify-between items-center bg-stone-50 dark:bg-stone-800/50 p-3 rounded-lg border border-stone-200/60 dark:border-stone-700/60 text-xs",
                                                    span { class: "text-gray-500 dark:text-gray-400", "Advance Balance" }
                                                    span { class: "text-base font-bold text-amber-600 dark:text-amber-400", "₹ {emp.current_balance:.2}" }
                                                }
                                            }
                                            CardFooter {
                                                div { class: "flex justify-end gap-2 w-full pt-1",
                                                    {
                                                        let e_edit = emp.clone();
                                                        let e_del_id = emp.id.clone();
                                                        rsx! {
                                                            Button {
                                                                variant: ButtonVariant::Outline,
                                                                onclick: move |_| {
                                                                    worker_form_id.set(Some(e_edit.id.clone()));
                                                                    worker_form_name.set(e_edit.name.clone());
                                                                    worker_form_role.set(e_edit.role.clone());
                                                                    worker_form_wage.set(e_edit.daily_wage.to_string());
                                                                    worker_form_balance.set(e_edit.current_balance.to_string());
                                                                    is_worker_sheet_open.set(true);
                                                                },
                                                                "{edit_str}"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: move |_| {
                                                                    worker_delete_id.set(Some(e_del_id.clone()));
                                                                },
                                                                "{delete_str}"
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
                                    icon: "👷",
                                    title: "No Workers Found",
                                    description: "No farm workers or staff members match your search criteria."
                                }
                            },
                            Some(Err(err)) => rsx! { ErrorState { message: err } },
                            None => rsx! { LoadingState {} }
                        }
                    } else if view_mode() == "SETTLEMENT" {
                        // PAYROLL SETTLEMENT VIEW
                        if !payroll_summaries.is_empty() && view_layout() == ViewLayout::Table {
                            // ── DESKTOP/TABLE VIEW: Payroll Settlement Table ──
                            div { class: "mt-2",
                                Card {
                                    div { class: "overflow-x-auto",
                                        table { class: "w-full text-sm text-left border-collapse",
                                            thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                tr {
                                                    th { class: "px-4 py-3", "Worker Name" }
                                                    th { class: "px-4 py-3 text-right", "Daily Wage" }
                                                    th { class: "px-4 py-3 text-right", "Working Days" }
                                                    th { class: "px-4 py-3 text-right", "Total Earned (₹)" }
                                                    th { class: "px-4 py-3 text-right", "Total Advances (₹)" }
                                                    th { class: "px-4 py-3 text-right", "Net Balance (₹)" }
                                                    th { class: "px-4 py-3 text-center", "Status" }
                                                }
                                            }
                                            tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                for summary in payroll_summaries.iter() {
                                                    tr {
                                                        key: "{summary.employee_name}",
                                                        class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                        td { class: "px-4 py-3 font-semibold text-stone-900 dark:text-stone-100 whitespace-nowrap", "{summary.employee_name}" }
                                                        td { class: "px-4 py-3 text-right font-medium text-stone-600 dark:text-stone-300 whitespace-nowrap", "₹{summary.daily_wage:.2}/day" }
                                                        td { class: "px-4 py-3 text-right font-medium text-stone-800 dark:text-stone-200 whitespace-nowrap", "{summary.total_attendance} Days" }
                                                        td { class: "px-4 py-3 text-right font-semibold text-emerald-600 dark:text-emerald-400 whitespace-nowrap", "₹{summary.total_earned:.2}" }
                                                        td { class: "px-4 py-3 text-right font-semibold text-amber-600 dark:text-amber-400 whitespace-nowrap", "₹{summary.total_advance:.2}" }
                                                        td {
                                                            class: if summary.net_balance < 0.0 {
                                                                "px-4 py-3 text-right font-bold text-rose-600 dark:text-rose-400 whitespace-nowrap"
                                                            } else {
                                                                "px-4 py-3 text-right font-bold text-emerald-600 dark:text-emerald-400 whitespace-nowrap"
                                                            },
                                                            "₹{summary.net_balance:.2}"
                                                        }
                                                        td { class: "px-4 py-3 text-center whitespace-nowrap",
                                                            span {
                                                                class: if summary.net_balance < 0.0 {
                                                                    "px-2.5 py-0.5 rounded text-xs font-semibold bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300 border border-amber-200 dark:border-amber-800"
                                                                } else {
                                                                    "px-2.5 py-0.5 rounded text-xs font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800"
                                                                },
                                                                if summary.net_balance < 0.0 { "Advance Overpayment" } else { "Payout Due" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                                tr {
                                                    td { class: "px-4 py-3", "TOTAL" }
                                                    td { class: "px-4 py-3 text-right text-stone-500 dark:text-stone-400 font-normal", "{payroll_summaries.len()} workers" }
                                                    td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{total_settlement_att:.1} Days" }
                                                    td { class: "px-4 py-3 text-right text-emerald-700 dark:text-emerald-300", "₹{total_settlement_earned:.2}" }
                                                    td { class: "px-4 py-3 text-right text-amber-700 dark:text-amber-300", "₹{total_settlement_adv:.2}" }
                                                    td { class: "px-4 py-3 text-right text-blue-700 dark:text-blue-300", "₹{total_settlement_net:.2}" }
                                                    td {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else if !payroll_summaries.is_empty() {
                            // ── CARD VIEW: Payroll Settlement Cards ──
                            div { class: "flex flex-col gap-4 mt-2",
                                for summary in payroll_summaries {
                                    Card { key: "{summary.employee_name}", class: "border-l-4 border-l-blue-500 dark:border-l-blue-400",
                                        CardHeader {
                                            div { class: "flex justify-between items-center",
                                                div {
                                                    h3 { class: "text-xl font-bold text-gray-900 dark:text-gray-100", "{summary.employee_name}" }
                                                    p { class: "text-xs text-gray-500 dark:text-gray-400", "Daily Wage Rate: ₹{summary.daily_wage:.2}/day" }
                                                }
                                                span {
                                                    class: if summary.net_balance < 0.0 {
                                                        "px-2.5 py-1 rounded-full text-xs font-bold bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300"
                                                    } else {
                                                        "px-2.5 py-1 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                                                    },
                                                    if summary.net_balance < 0.0 { "Advance Overpayment" } else { "Payout Due" }
                                                }
                                            }
                                        }
                                        CardContent {
                                            div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3 py-2 bg-stone-50 dark:bg-stone-800/50 rounded-lg p-3 border border-stone-200/60 dark:border-stone-700/60 text-sm",
                                                div {
                                                    p { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "Working Days" }
                                                    p { class: "text-base font-bold text-gray-900 dark:text-gray-100 mt-0.5", "{summary.total_attendance} Days" }
                                                }
                                                div {
                                                    p { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "Earned Amount" }
                                                    p { class: "text-base font-bold text-emerald-600 dark:text-emerald-400 mt-0.5", "₹{summary.total_earned:.2}" }
                                                }
                                                div {
                                                    p { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "Total Advances" }
                                                    p { class: "text-base font-bold text-amber-600 dark:text-amber-400 mt-0.5", "₹{summary.total_advance:.2}" }
                                                }
                                                div {
                                                    p { class: "text-xs text-gray-500 dark:text-gray-400 font-medium", "Net Balance" }
                                                    p {
                                                        class: if summary.net_balance < 0.0 { "text-base font-bold text-rose-600 dark:text-rose-400 mt-0.5" } else { "text-base font-bold text-emerald-600 dark:text-emerald-400 mt-0.5" },
                                                        "₹{summary.net_balance:.2}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            EmptyState {
                                icon: "🧮",
                                title: "No Settlement Summaries",
                                description: "No labor attendance records match your search or date filter to calculate payroll settlement."
                            }
                        }
                    } else {
                        // DAILY LOGS VIEW
                        match records.cloned() {
                            Some(Ok(_)) if !filtered_records.is_empty() && view_layout() == ViewLayout::Table => rsx! {
                                // ── DESKTOP/TABLE VIEW: Daily Logs Data Table ──
                                div { class: "mt-2",
                                    Card {
                                        div { class: "overflow-x-auto",
                                            table { class: "w-full text-sm text-left border-collapse",
                                                thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                    tr {
                                                        th { class: "px-4 py-3", "Date" }
                                                        th { class: "px-4 py-3", "Employee Name" }
                                                        th { class: "px-4 py-3 text-right", "Attendance (Days)" }
                                                        th { class: "px-4 py-3 text-right", "Advance Given (₹)" }
                                                        th { class: "px-4 py-3 text-right", "Actions" }
                                                    }
                                                }
                                                tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                    for item in filtered_records.iter() {
                                                        {
                                                            let edit_item = item.clone();
                                                            let del_id = item.id.clone();
                                                            rsx! {
                                                                tr {
                                                                    key: "{item.id}",
                                                                    class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                    td { class: "px-4 py-3 font-medium text-stone-900 dark:text-stone-100 whitespace-nowrap", "{item.date}" }
                                                                    td { class: "px-4 py-3 font-semibold text-blue-600 dark:text-blue-400 whitespace-nowrap", "{item.employee_name}" }
                                                                    td { class: "px-4 py-3 text-right font-medium text-stone-800 dark:text-stone-200 whitespace-nowrap",
                                                                        span { class: "px-2 py-0.5 rounded bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 font-semibold text-xs", "{item.attendance} Day(s)" }
                                                                    }
                                                                    td { class: "px-4 py-3 text-right font-bold text-amber-600 dark:text-amber-400", "₹{item.advance_given:.2}" }
                                                                    td { class: "px-4 py-3 text-right whitespace-nowrap",
                                                                        div { class: "flex items-center justify-end gap-2",
                                                                            Button {
                                                                                variant: ButtonVariant::Outline,
                                                                                onclick: move |_| {
                                                                                    labor_form_id.set(Some(edit_item.id.clone()));
                                                                                    form_date.set(edit_item.date.clone());
                                                                                    form_employee_name.set(edit_item.employee_name.clone());
                                                                                    form_employee_id.set(edit_item.employee_id.clone());
                                                                                    form_attendance.set(edit_item.attendance.to_string());
                                                                                    form_advance.set(edit_item.advance_given.to_string());
                                                                                    is_labor_sheet_open.set(true);
                                                                                },
                                                                                "{edit_str}"
                                                                            }
                                                                            Button {
                                                                                variant: ButtonVariant::Destructive,
                                                                                onclick: move |_| {
                                                                                    labor_delete_id.set(Some(del_id.clone()));
                                                                                },
                                                                                "{delete_str}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                tfoot { class: "bg-amber-50/70 dark:bg-stone-800/90 font-bold border-t border-amber-200 dark:border-stone-700 text-stone-900 dark:text-stone-100",
                                                    tr {
                                                        td { class: "px-4 py-3", "TOTAL" }
                                                        td { class: "px-4 py-3 text-stone-500 dark:text-stone-400 font-normal", "{filtered_records.len()} logs" }
                                                        td { class: "px-4 py-3 text-right text-stone-900 dark:text-stone-100", "{total_att_sum:.1} Days" }
                                                        td { class: "px-4 py-3 text-right text-amber-700 dark:text-amber-300", "₹{total_adv_sum:.2}" }
                                                        td {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Ok(_)) if !filtered_records.is_empty() => rsx! {
                                // ── CARD VIEW: Daily Logs Cards ──
                                div { class: "flex flex-col gap-3 mt-2",
                                    for item in filtered_records {
                                        Card { key: "{item.id}",
                                            CardHeader {
                                                div { class: "flex justify-between items-center text-xs text-gray-500 dark:text-gray-400 font-medium",
                                                    span { "🗓️ {item.date}" }
                                                    span { class: "px-2 py-0.5 rounded bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 font-semibold", "Attendance: {item.attendance} Day(s)" }
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
                                                                    labor_form_id.set(Some(edit_item.id.clone()));
                                                                    form_date.set(edit_item.date.clone());
                                                                    form_employee_name.set(edit_item.employee_name.clone());
                                                                    form_employee_id.set(edit_item.employee_id.clone());
                                                                    form_attendance.set(edit_item.attendance.to_string());
                                                                    form_advance.set(edit_item.advance_given.to_string());
                                                                    is_labor_sheet_open.set(true);
                                                                },
                                                                "{edit_str}"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: move |_| {
                                                                    labor_delete_id.set(Some(del_id.clone()));
                                                                },
                                                                "{delete_str}"
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
                    }

                    // Labor Delete Confirm Dialog
                    ConfirmDialog {
                        is_open: labor_delete_id().is_some(),
                        is_deleting: is_labor_deleting(),
                        title: "Delete Labor Record".to_string(),
                        description: "Are you sure you want to delete this labor record? Employee balance will automatically update.".to_string(),
                        onconfirm: confirm_delete_labor,
                        oncancel: move |_| {
                            if !is_labor_deleting() {
                                labor_delete_id.set(None);
                            }
                        }
                    }

                    // Worker Delete Confirm Dialog
                    ConfirmDialog {
                        is_open: worker_delete_id().is_some(),
                        is_deleting: is_worker_deleting(),
                        title: "Delete Worker Profile".to_string(),
                        description: "Are you sure you want to delete this worker? This will not delete their past attendance logs.".to_string(),
                        onconfirm: confirm_delete_worker,
                        oncancel: move |_| {
                            if !is_worker_deleting() {
                                worker_delete_id.set(None);
                            }
                        }
                    }
                }
            }

            // --- LABOR SHEET ---
            if is_labor_sheet_open() {
                Sheet {
                    open: Some(is_labor_sheet_open()),
                    on_open_change: move |open| is_labor_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if labor_form_id().is_some() { "Edit Labor Record" } else { "Log Labor & Advance" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !labor_form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{labor_form_error}" }
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
                            Label { html_for: "labor-emp", "Employee / Worker" }
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
                            disabled: is_labor_submitting(),
                            onclick: move |_| is_labor_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            disabled: is_labor_submitting(),
                            loading: is_labor_submitting(),
                            onclick: submit_labor_handler,
                            if labor_form_id().is_some() { "Update Record" } else { "Save Record" }
                        }
                    }
                }
            }

            // --- WORKER SHEET ---
            if is_worker_sheet_open() {
                Sheet {
                    open: Some(is_worker_sheet_open()),
                    on_open_change: move |open| is_worker_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if worker_form_id().is_some() { "Edit Worker Profile" } else { "Add New Worker" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !worker_form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{worker_form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "emp-name", "Worker Name" }
                            Input {
                                placeholder: "e.g. Ramesh Kumar",
                                value: "{worker_form_name}",
                                oninput: move |e: Event<FormData>| worker_form_name.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "emp-role", "Role / Designation" }
                            Input {
                                placeholder: "e.g. Farm Worker, Shed Manager, Driver",
                                value: "{worker_form_role}",
                                oninput: move |e: Event<FormData>| worker_form_role.set(e.value())
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "emp-wage", "Daily Wage Rate (₹)" }
                                Input {
                                    r#type: "number",
                                    step: "1",
                                    value: "{worker_form_wage}",
                                    oninput: move |e: Event<FormData>| worker_form_wage.set(e.value())
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "emp-balance", "Starting Advance Balance (₹)" }
                                Input {
                                    r#type: "number",
                                    step: "1",
                                    value: "{worker_form_balance}",
                                    oninput: move |e: Event<FormData>| worker_form_balance.set(e.value())
                                }
                            }
                        }
                    }
                    SheetFooter {
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: is_worker_submitting(),
                            onclick: move |_| is_worker_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            disabled: is_worker_submitting(),
                            loading: is_worker_submitting(),
                            onclick: submit_worker_handler,
                            if worker_form_id().is_some() { "Update Worker" } else { "Save Worker" }
                        }
                    }
                }
            }
        }
    }
}
