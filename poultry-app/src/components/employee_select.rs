use dioxus::prelude::*;
use crate::services::use_employees;

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeSelectProps {
    pub value: String,
    pub onchange: EventHandler<(String, String)>, // (employee_id, employee_name)
    #[props(default = false)]
    pub required: bool,
    #[props(default = "Select an employee...".to_string())]
    pub placeholder: String,
}

#[component]
pub fn EmployeeSelect(props: EmployeeSelectProps) -> Element {
    let employees_res = use_employees();
    let employees = employees_res.cloned().and_then(|r| r.ok()).unwrap_or_default();
    let employees_for_onchange = employees.clone();

    rsx! {
        select {
            class: "w-full flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 dark:bg-stone-800 dark:border-stone-700 dark:text-gray-100",
            value: "{props.value}",
            required: props.required,
            onchange: move |evt: Event<FormData>| {
                let selected_val = evt.value();
                if let Some(matched) = employees_for_onchange.iter().find(|e| e.id == selected_val || e.name == selected_val) {
                    props.onchange.call((matched.id.clone(), matched.name.clone()));
                } else {
                    props.onchange.call((String::new(), selected_val));
                }
            },
            option { value: "", disabled: props.required, "{props.placeholder}" }
            for e in employees {
                option {
                    key: "{e.id}",
                    value: "{e.id}",
                    selected: props.value == e.id || props.value == e.name,
                    "{e.name} ({e.role})"
                }
            }
        }
    }
}
