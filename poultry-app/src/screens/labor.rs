use dioxus::prelude::*;
use chrono::Utc;
use crate::components::toast::{use_toast, ToastOptions};

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::employee_select::EmployeeSelect;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::month_filter::MonthFilter;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::{LaborRecord, LaborPayrollSummary};
use crate::services::*;

#[component]
pub fn Labor() -> Element {
    let mut records = use_labor_records();
    let employees_res = use_employees();
    let api = use_auth();
    let toast_api = use_toast();
    let mut is_sheet_open = use_signal(|| false);

    let current_month_str = Utc::now().format("%Y-%m").to_string();
    let mut selected_month = use_signal(move || Some(current_month_str.clone()));
    let mut search_query = use_signal(String::new);
    let mut view_mode = use_signal(|| "LOGS".to_string()); // "LOGS" or "SETTLEMENT"

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let today_str = Utc::now().format("%Y-%m-%d").to_string();
    let mut form_date = use_signal(move || today_str.clone());
    let mut form_employee_name = use_signal(String::new);
    let mut form_employee_id = use_signal(|| Option::<String>::None);
    let mut form_attendance = use_signal(|| "1.0".to_string());
    let mut form_advance = use_signal(|| "0.0".to_string());
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

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

        form_error.set(String::new());
        is_submitting.set(true);

        let is_edit = form_id().is_some();
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

            is_submitting.set(false);

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_error.set(String::new());
                    records.restart();
                    let desc = if is_edit { "Labor record updated successfully." } else { "Labor record logged successfully." };
                    toast_api.success("Success".to_string(), ToastOptions::new().description(desc));
                }
                Err(e) => {
                    form_error.set(e.clone());
                    toast_api.error("Error".to_string(), ToastOptions::new().description(e));
                }
            }
        });
    };

    let confirm_delete = move |_| {
        if is_deleting() {
            return;
        }
        if let Some(id) = delete_id() {
            is_deleting.set(true);
            spawn(async move {
                let res = api.delete(&format!("/api/labor/{}", id)).await;
                if let Ok(_) = res {
                    delete_id.set(None);
                    records.restart();
                    toast_api.success("Success".to_string(), ToastOptions::new().description("Labor record deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error("Error".to_string(), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let rec_list = records.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let emp_list = employees_res.cloned().and_then(|r| r.ok()).unwrap_or_default();

    let q = search_query().trim().to_lowercase();
    let filtered_records: Vec<_> = rec_list.iter().filter(|r| {
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
    }).cloned().collect();

    // Calculate Payroll Summary per Employee
    let mut payroll_summaries: Vec<LaborPayrollSummary> = Vec::new();
    let mut processed_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for r in &filtered_records {
        if processed_names.contains(&r.employee_name) {
            continue;
        }
        processed_names.insert(r.employee_name.clone());

        // Sum attendance and advances for this worker in filtered range
        let worker_records: Vec<&LaborRecord> = filtered_records
            .iter()
            .filter(|rec| rec.employee_name == r.employee_name)
            .collect();

        let total_att: f64 = worker_records.iter().map(|rec| rec.attendance).sum();
        let total_adv: f64 = worker_records.iter().map(|rec| rec.advance_given).sum();

        // Match daily wage from employee directory or default 500
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

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",

            // FIXED HEADER
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-3",
                div { class: "flex justify-between items-center",
                    div {
                        h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Labor & Attendance Management" }
                        p { class: "text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Track daily attendance, cash advances & monthly wage settlements" }
                    }
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
                        "+ Log Record"
                    }
                }

                // Mode Toggle Bar (Logs vs Settlement) & Filters
                div { class: "flex flex-col sm:flex-row justify-between items-stretch sm:items-center gap-3 pt-1",
                    div { class: "flex p-1 bg-stone-100 dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 text-xs font-medium self-start",
                        button {
                            class: if view_mode() == "LOGS" {
                                "px-3 py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all flex items-center gap-1"
                            } else {
                                "px-3 py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1"
                            },
                            onclick: move |_| view_mode.set("LOGS".to_string()),
                            "📋 Daily Logs"
                        }
                        button {
                            class: if view_mode() == "SETTLEMENT" {
                                "px-3 py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all flex items-center gap-1"
                            } else {
                                "px-3 py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all flex items-center gap-1"
                            },
                            onclick: move |_| view_mode.set("SETTLEMENT".to_string()),
                            "🧮 Monthly Payroll Settlement"
                        }
                    }

                    div { class: "flex items-center gap-2",
                        MonthFilter {
                            selected: selected_month(),
                            onchange: move |m| selected_month.set(m)
                        }
                    }
                }

                Input {
                    placeholder: "🔍 Search employee name or date...",
                    value: "{search_query}",
                    oninput: move |e: Event<FormData>| search_query.set(e.value())
                }
            }

            // SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-3xl mx-auto pb-20",

                    if view_mode() == "SETTLEMENT" {
                        // PAYROLL SETTLEMENT VIEW
                        if !payroll_summaries.is_empty() {
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
                            Some(Ok(_)) if !filtered_records.is_empty() => rsx! {
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
                    }

                    ConfirmDialog {
                        is_open: delete_id().is_some(),
                        is_deleting: is_deleting(),
                        title: "Delete Labor Record".to_string(),
                        description: "Are you sure you want to delete this labor record? Employee balance will automatically update.".to_string(),
                        onconfirm: confirm_delete,
                        oncancel: move |_| {
                            if !is_deleting() {
                                delete_id.set(None);
                            }
                        }
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
                            disabled: is_submitting(),
                            onclick: move |_| is_sheet_open.set(false),
                            "Cancel"
                        }
                        Button {
                            disabled: is_submitting(),
                            loading: is_submitting(),
                            onclick: submit_handler,
                            if form_id().is_some() { "Update Record" } else { "Save Record" }
                        }
                    }
                }
            }
        }
    }
}
