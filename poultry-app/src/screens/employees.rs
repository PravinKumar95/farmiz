use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::Employee;
use crate::services::*;

#[component]
pub fn Employees() -> Element {
    let mut employees = use_employees();
    let api = use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let mut form_name = use_signal(String::new);
    let mut form_role = use_signal(String::new);
    let mut form_wage = use_signal(|| "0.0".to_string());
    let mut form_balance = use_signal(|| "0.0".to_string());
    let mut form_error = use_signal(String::new);

    let submit_handler = move |_| {
        let name = form_name().trim().to_string();
        let role = form_role().trim().to_string();
        let wage_str = form_wage().trim().to_string();
        let balance_str = form_balance().trim().to_string();

        if name.len() < 2 {
            form_error.set("Name must be at least 2 characters long.".to_string());
            return;
        }
        if role.is_empty() {
            form_error.set("Role is required.".to_string());
            return;
        }
        let daily_wage: f64 = match wage_str.parse() {
            Ok(w) => w,
            Err(_) => {
                form_error.set("Daily wage must be a valid number.".to_string());
                return;
            }
        };
        let current_balance: f64 = match balance_str.parse() {
            Ok(b) => b,
            Err(_) => {
                form_error.set("Balance must be a valid number.".to_string());
                return;
            }
        };

        form_error.set(String::new());
        let new_employee = Employee {
            id: form_id().unwrap_or_default(),
            name,
            role,
            daily_wage,
            current_balance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/employees/{}", id), &new_employee).await
            } else {
                api.post("/api/employees", &new_employee).await
            };

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_name.set(String::new());
                    form_role.set(String::new());
                    form_wage.set("0.0".to_string());
                    form_balance.set("0.0".to_string());
                    employees.restart();
                }
                Err(e) => form_error.set(e),
            }
        });
    };

    let confirm_delete = move |_| {
        if let Some(id) = delete_id() {
            spawn(async move {
                if let Ok(_) = api.delete(&format!("/api/employees/{}", id)).await {
                    delete_id.set(None);
                    employees.restart();
                }
            });
        }
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-4xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Employees Directory" }
                Button {
                    onclick: move |_| {
                        form_id.set(None);
                        form_name.set(String::new());
                        form_role.set("Farm Worker".to_string());
                        form_wage.set("500.0".to_string());
                        form_balance.set("0.0".to_string());
                        form_error.set(String::new());
                        is_sheet_open.set(true);
                    },
                    "Add Employee"
                }
            }

            match employees.cloned() {
                Some(Ok(list)) if !list.is_empty() => rsx! {
                    Card { class: "mt-4",
                        CardContent { class: "p-0 overflow-hidden rounded-xl",
                            div { class: "overflow-x-auto",
                                table { class: "w-full text-sm text-left",
                                    thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-border",
                                        tr {
                                            th { class: "px-6 py-3", "Employee Name" }
                                            th { class: "px-6 py-3", "Role" }
                                            th { class: "px-6 py-3 text-right", "Daily Wage" }
                                            th { class: "px-6 py-3 text-right", "Advance Balance" }
                                            th { class: "px-6 py-3 text-right", "Actions" }
                                        }
                                    }
                                    tbody { class: "divide-y divide-border",
                                        for emp in list {
                                            tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800 transition-colors",
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
                                                                    form_id.set(Some(e_edit.id.clone()));
                                                                    form_name.set(e_edit.name.clone());
                                                                    form_role.set(e_edit.role.clone());
                                                                    form_wage.set(e_edit.daily_wage.to_string());
                                                                    form_balance.set(e_edit.current_balance.to_string());
                                                                    is_sheet_open.set(true);
                                                                },
                                                                "Edit"
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Destructive,
                                                                onclick: move |_| {
                                                                    delete_id.set(Some(e_del_id.clone()));
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
                            }
                        }
                    }
                },
                Some(Ok(_)) => rsx! {
                    EmptyState {
                        icon: "👷",
                        title: "No Employees Registered",
                        description: "No farm staff or laborers registered yet."
                    }
                },
                Some(Err(err)) => rsx! { ErrorState { message: err } },
                None => rsx! { LoadingState {} }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { if form_id().is_some() { "Edit Employee" } else { "Add New Employee" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "emp-name", "Employee Name" }
                            Input {
                                placeholder: "e.g. Ramesh Kumar",
                                value: "{form_name}",
                                oninput: move |e: Event<FormData>| form_name.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "emp-role", "Role" }
                            Input {
                                placeholder: "e.g. Shed Manager, Laborer, Driver",
                                value: "{form_role}",
                                oninput: move |e: Event<FormData>| form_role.set(e.value())
                            }
                        }
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "emp-wage", "Daily Wage (₹)" }
                                Input {
                                    r#type: "number",
                                    step: "1",
                                    value: "{form_wage}",
                                    oninput: move |e: Event<FormData>| form_wage.set(e.value())
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "emp-bal", "Current Balance (₹)" }
                                Input {
                                    r#type: "number",
                                    step: "0.01",
                                    value: "{form_balance}",
                                    oninput: move |e: Event<FormData>| form_balance.set(e.value())
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
                            if form_id().is_some() { "Update Employee" } else { "Save Employee" }
                        }
                    }
                }
            }

            ConfirmDialog {
                is_open: delete_id().is_some(),
                title: "Delete Employee".to_string(),
                description: "Are you sure you want to delete this employee? Record will be permanently removed.".to_string(),
                onconfirm: confirm_delete,
                oncancel: move |_| delete_id.set(None)
            }
        }
    }
}
