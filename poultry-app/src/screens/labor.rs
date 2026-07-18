use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardHeader, CardFooter};
use crate::components::sheet::{Sheet, SheetHeader, SheetTitle, SheetFooter};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::services::*;
use crate::models::LaborRecord;

#[component]
pub fn Labor() -> Element {
    let mut employees = use_employees();
    let mut records = use_labor_records();
    let api = crate::services::use_auth();
    let mut is_sheet_open = use_signal(|| false);

    let mut form_id = use_signal(|| Option::<String>::None);

    let mut form_date = use_signal(|| String::new());
    let mut form_employee = use_signal(|| String::new());
    let mut form_attendance = use_signal(|| String::new());
    let mut form_advance = use_signal(|| String::new());
    let mut form_error = use_signal(|| String::new());

    let submit_handler = move |_| {
        let date = form_date().trim().to_string();
        let employee = form_employee().trim().to_string();
        
        if date.is_empty() { form_error.set("Date required".to_string()); return; }
        if employee.is_empty() { form_error.set("Employee name required".to_string()); return; }
        
        let attendance: f64 = match form_attendance().parse() { Ok(v) => v, Err(_) => { form_error.set("Invalid attendance".to_string()); return; } };
        let advance: f64 = match form_advance().parse() { Ok(v) => v, Err(_) => { form_error.set("Invalid advance amount".to_string()); return; } };
        
        let mut new_record = LaborRecord {
            id: form_id().unwrap_or_default(),
            date,
            employee_name: employee,
            attendance,
            advance_given: advance,
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

    rsx! {
        div { class: "flex flex-col gap-4 w-full max-w-2xl mx-auto pb-20",
            datalist { id: "employees-list",
                for emp in employees.cloned().unwrap_or_default() {
                    option { value: "{emp.name}" }
                }
            }
            div { class: "flex justify-between items-center",
                h1 { class: "text-2xl font-bold tracking-tight", "Labor Management" }
                Button { 
                    onclick: move |_| {
                        form_id.set(None);
                        form_date.set(String::new());
                        form_employee.set(String::new());
                        form_attendance.set(String::new());
                        form_advance.set(String::new());
                        is_sheet_open.set(true);
                    }, 
                    "Add Record" 
                }
            }

            if is_sheet_open() {
                Sheet {
                    open: Some(is_sheet_open()),
                    on_open_change: move |open| is_sheet_open.set(open),
                    SheetHeader { SheetTitle { "Add Labor Record" } }
                    div { class: "flex flex-col gap-4 py-4 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "text-sm text-red-500 font-medium", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "date", "Date" }
                            Input { value: "{form_date}", oninput: move |e: FormEvent| form_date.set(e.value()), placeholder: "YYYY-MM-DD" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "emp", "Employee Name" }
                            Input { list: "employees-list", value: "{form_employee}", oninput: move |e: FormEvent| form_employee.set(e.value()) }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "att", "Attendance (Days)" }
                            Input { value: "{form_attendance}", oninput: move |e: FormEvent| form_attendance.set(e.value()), placeholder: "1.0" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "adv", "Advance Given" }
                            Input { value: "{form_advance}", oninput: move |e: FormEvent| form_advance.set(e.value()), placeholder: "0.00" }
                        }
                    }
                    SheetFooter {
                        Button { onclick: submit_handler, "Save Record" }
                        Button { onclick: move |_| is_sheet_open.set(false), "Cancel" }
                    }
                }
            }

            div { class: "flex flex-col gap-3 mt-4",
                for record in records.cloned().unwrap_or_default() {
                    Card { key: "{record.id}",
                        CardHeader {
                            div { class: "flex justify-between items-center text-sm",
                                span { class: "text-muted-foreground", "{record.date}" }
                                span { class: "font-semibold text-primary", "{record.employee_name}" }
                            }
                        }
                        CardContent {
                            div { class: "flex justify-between items-center",
                                div { class: "flex flex-col gap-1",
                                    span { class: "text-xs text-muted-foreground uppercase tracking-wider", "Attendance" }
                                    span { class: "font-medium text-lg", "{record.attendance} Days" }
                                }
                                div { class: "flex flex-col gap-1 items-end",
                                    span { class: "text-xs text-muted-foreground uppercase tracking-wider", "Advance Given" }
                                    span { class: "font-bold text-lg text-red-500", "₹ {record.advance_given:.2}" }
                                }
                            }
                        }
                        CardFooter { class: "bg-muted/50 pt-4 rounded-b-xl flex justify-end gap-2 w-full text-sm",
                            {
                                let edit_record = record.clone();
                                let delete_id = record.id.clone();
                                rsx! {
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {
                                            form_id.set(Some(edit_record.id.clone()));
                                            form_date.set(edit_record.date.clone());
                                            form_employee.set(edit_record.employee_name.clone());
                                            form_attendance.set(edit_record.attendance.to_string());
                                            form_advance.set(edit_record.advance_given.to_string());
                                            is_sheet_open.set(true);
                                        },
                                        "Edit"
                                    }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {
                                            let id = delete_id.clone();
                                            spawn(async move {
                                                let _ = api.delete(&format!("/api/labor/{}", id)).await;
                                                records.restart();
                                            });
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
