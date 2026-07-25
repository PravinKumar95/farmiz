use dioxus::prelude::*;
use crate::components::button::Button;
use crate::components::card::{Card, CardContent};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::services::*;
use crate::models::Employee;
use crate::components::empty_state::{EmptyState, LoadingState, ErrorState};

#[component]
pub fn Employees() -> Element {
    let mut employees = use_employees();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);
    
    // Form state
    let mut form_name = use_signal(String::new);
    let mut form_role = use_signal(String::new);
    let mut form_wage = use_signal(String::new);
    let mut form_balance = use_signal(String::new);
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
            id: String::new(),
            name,
            role,
            daily_wage,
            current_balance,
            created_at: None,
        };

        spawn(async move {
            match api.post("/api/employees", &new_employee).await {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_name.set(String::new());
                    form_role.set(String::new());
                    form_wage.set(String::new());
                    form_balance.set(String::new());
                    employees.restart();
                }
                Err(e) => {
                    form_error.set(e);
                }
            }
        });
    };

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "Employees Directory" }
                Button { 
                    onclick: move |_| is_sheet_open.set(true),
                    "Add Employee" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader {
                        SheetTitle { "Add New Employee" }
                    }
                    div { class: "flex flex-col gap-4 px-6 py-4",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "name", "Name" }
                            Input {
                                value: "{form_name}",
                                oninput: move |e: FormEvent| form_name.set(e.value()),
                                placeholder: "e.g., Ramesh"
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "role", "Role" }
                            Input {
                                value: "{form_role}",
                                oninput: move |e: FormEvent| form_role.set(e.value()),
                                placeholder: "e.g., Laborer"
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "wage", "Daily Wage" }
                            Input {
                                value: "{form_wage}",
                                oninput: move |e: FormEvent| form_wage.set(e.value()),
                                placeholder: "0.00"
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "balance", "Opening Balance" }
                            Input {
                                value: "{form_balance}",
                                oninput: move |e: FormEvent| form_balance.set(e.value()),
                                placeholder: "0.00"
                            }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, "Save Employee" }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            match employees.cloned() {
                Some(Ok(list)) if !list.is_empty() => rsx! {
                    div { class: "flex flex-col gap-3 mt-4",
                        for employee in list {
                            div {
                                key: "{employee.id}",
                                class: "block rounded-xl",
                                Card {
                                    CardContent {
                                        div { class: "flex justify-between items-center",
                                            div { class: "flex flex-col",
                                                span { class: "font-bold text-base text-gray-900 dark:text-gray-100", "{employee.name}" }
                                                span { class: "text-xs text-gray-500 dark:text-gray-400 mt-1 uppercase tracking-wider", "{employee.role}" }
                                            }
                                            div { class: "flex flex-col items-end",
                                                div { class: "text-xs text-gray-500 dark:text-gray-400 mb-1", "Current Balance" }
                                                div {
                                                    class: if employee.current_balance < 0.0 {
                                                        "font-bold text-lg text-red-500"
                                                    } else {
                                                        "font-bold text-lg text-green-600"
                                                    },
                                                    "₹ {employee.current_balance.abs():.2}"
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
                        title: "No Employees Found",
                        description: "No employee records found. Click 'Add New Employee' above to add an employee."
                    }
                },
                Some(Err(err)) => rsx! {
                    ErrorState {
                        message: err,
                        on_retry: move |_| { employees.restart(); }
                    }
                },
                None => rsx! {
                    LoadingState {}
                }
            }
        }
    }
}
