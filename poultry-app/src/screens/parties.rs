use dioxus::prelude::*;
use dioxus_router::components::Link;
use crate::components::toast::{use_toast, ToastOptions};
use crate::i18n::tr;

use crate::components::button::{Button, ButtonVariant};
use crate::components::card::{Card, CardContent, CardFooter, CardHeader};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::empty_state::{EmptyState, ErrorState, LoadingState};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::layout_toggle::{LayoutToggle, ViewLayout};
use crate::components::sheet::{Sheet, SheetFooter, SheetHeader, SheetTitle};
use crate::models::Party;
use crate::services::*;

#[component]
pub fn Parties() -> Element {
    let mut parties = use_parties();
    let api = use_auth();
    let toast_api = use_toast();
    let mut view_layout = use_signal(ViewLayout::default);
    let mut is_sheet_open = use_signal(|| false);

    let mut delete_id = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| Option::<String>::None);

    let mut selected_tab = use_signal(|| "ALL".to_string());
    let mut search_query = use_signal(String::new);

    let mut form_name = use_signal(String::new);
    let mut form_type = use_signal(String::new);
    let mut form_balance = use_signal(|| "0.0".to_string());
    let mut form_error = use_signal(String::new);

    let mut is_submitting = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    let submit_handler = move |_| {
        if is_submitting() {
            return;
        }

        let name = form_name().trim().to_string();
        let p_type = form_type().trim().to_uppercase();
        let balance_str = form_balance().trim().to_string();

        if name.len() < 2 {
            form_error.set("Name must be at least 2 characters long.".to_string());
            return;
        }
        let allowed_types = ["CUSTOMER", "SUPPLIER", "BAKERY"];
        if !allowed_types.contains(&p_type.as_str()) {
            form_error.set("Type must be CUSTOMER, SUPPLIER, or BAKERY.".to_string());
            return;
        }
        let balance: f64 = match balance_str.parse() {
            Ok(b) => b,
            Err(_) => {
                form_error.set("Balance must be a valid number.".to_string());
                return;
            }
        };

        form_error.set(String::new());
        is_submitting.set(true);

        let is_edit = form_id().is_some();
        let party = Party {
            id: form_id().unwrap_or_default(),
            name,
            party_type: p_type,
            current_balance: balance,
            user_id: None,
            created_at: None,
        };

        spawn(async move {
            let res = if let Some(id) = form_id() {
                api.put(&format!("/api/parties/{}", id), &party).await
            } else {
                api.post("/api/parties", &party).await
            };

            is_submitting.set(false);

            match res {
                Ok(_) => {
                    is_sheet_open.set(false);
                    form_name.set(String::new());
                    form_type.set(String::new());
                    form_balance.set("0.0".to_string());
                    parties.restart();
                    let desc = if is_edit { "Party updated successfully." } else { "Party created successfully." };
                    toast_api.success(tr("success"), ToastOptions::new().description(desc));
                }
                Err(e) => {
                    form_error.set(e.clone());
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
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
                let res = api.delete(&format!("/api/parties/{}", id)).await;
                if let Ok(_) = res {
                    delete_id.set(None);
                    parties.restart();
                    toast_api.success(tr("success"), ToastOptions::new().description("Party deleted successfully."));
                } else if let Err(e) = res {
                    toast_api.error(tr("error"), ToastOptions::new().description(e));
                }
                is_deleting.set(false);
            });
        }
    };

    let party_list = parties.cloned().and_then(|r| r.ok()).unwrap_or_default();

    // Filter list by selected tab & search query
    let q = search_query().trim().to_lowercase();
    let current_tab = selected_tab();
    let filtered_list: Vec<_> = party_list
        .into_iter()
        .filter(|p| {
            let matches_tab = if current_tab == "ALL" {
                true
            } else {
                p.party_type == current_tab
            };
            let matches_search = if q.is_empty() {
                true
            } else {
                p.name.to_lowercase().contains(&q) || p.party_type.to_lowercase().contains(&q)
            };
            matches_tab && matches_search
        })
        .collect();

    let title_str = tr("ledger");
    let add_btn_str = tr("add-party");
    let search_ph = tr("search-placeholder");
    let edit_str = tr("edit");
    let delete_str = tr("delete");

    rsx! {
        div { class: "flex flex-col h-full max-h-full w-full min-h-0 overflow-hidden",

            // SECTION 1: FIXED HEADER — does NOT scroll
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-3",
                div { class: "flex justify-between items-center",
                    div {
                        h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "📒 {title_str}" }
                        p { class: "text-xs text-gray-500 dark:text-gray-400 mt-0.5", "Manage customer receivables, supplier payables & ledger balances" }
                    }
                    Button {
                        onclick: move |_| {
                            form_id.set(None);
                            form_name.set(String::new());
                            form_type.set("CUSTOMER".to_string());
                            form_balance.set("0.0".to_string());
                            form_error.set(String::new());
                            is_sheet_open.set(true);
                        },
                        "+ {add_btn_str}"
                    }
                }

                // Controls: Filter Tabs, Search Bar & Layout Toggle
                div { class: "flex flex-col sm:flex-row justify-between items-stretch sm:items-center gap-3 pt-1",
                    div { class: "flex flex-wrap gap-1 p-1 bg-stone-100 dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 text-xs font-medium self-start",
                        for (tab_key, tab_label) in [("ALL", "All"), ("CUSTOMER", "Customers"), ("SUPPLIER", "Suppliers"), ("BAKERY", "Bakeries")] {
                            button {
                                key: "{tab_key}",
                                class: if selected_tab() == tab_key {
                                    "px-3 py-1.5 rounded-md bg-white dark:bg-stone-900 font-semibold text-blue-600 dark:text-blue-400 shadow-sm transition-all"
                                } else {
                                    "px-3 py-1.5 rounded-md text-stone-600 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-200 transition-all"
                                },
                                onclick: move |_| selected_tab.set(tab_key.to_string()),
                                "{tab_label}"
                            }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        div { class: "w-full sm:w-64",
                            Input {
                                placeholder: "{search_ph}",
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
            }

            // SECTION 2: SCROLLABLE CONTENT AREA
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-4 w-full max-w-5xl mx-auto pb-20",
                    match parties.cloned() {
                        Some(Ok(_)) if !filtered_list.is_empty() && view_layout() == ViewLayout::Table => rsx! {
                            Card { class: "mt-2",
                                CardContent { class: "p-0 rounded-xl",
                                    div { class: "overflow-x-auto",
                                        table { class: "w-full text-sm text-left",
                                            thead { class: "text-xs text-gray-500 uppercase bg-gray-50 dark:bg-stone-800 border-b border-stone-200 dark:border-stone-700",
                                                tr {
                                                    th { class: "px-6 py-3", "Party Name" }
                                                    th { class: "px-6 py-3", "Type" }
                                                    th { class: "px-6 py-3 text-right", "Current Balance" }
                                                    th { class: "px-6 py-3 text-right", "Actions" }
                                                }
                                            }
                                            tbody { class: "divide-y divide-stone-200 dark:divide-stone-800",
                                                for party in filtered_list {
                                                    {
                                                        let bal_str = if party.party_type == "SUPPLIER" {
                                                            if party.current_balance < 0.0 {
                                                                format!("-₹ {:.2} (Payable)", party.current_balance.abs())
                                                            } else if party.current_balance > 0.0 {
                                                                format!("+₹ {:.2} (Surplus)", party.current_balance)
                                                            } else {
                                                                "₹ 0.00".to_string()
                                                            }
                                                        } else {
                                                            if party.current_balance > 0.0 {
                                                                format!("₹ {:.2} (Due)", party.current_balance)
                                                            } else if party.current_balance < 0.0 {
                                                                format!("-₹ {:.2} (Overpaid)", party.current_balance.abs())
                                                            } else {
                                                                "₹ 0.00".to_string()
                                                            }
                                                        };
                                                        let p_edit = party.clone();
                                                        let p_del_id = party.id.clone();
                                                        rsx! {
                                                            tr { class: "hover:bg-gray-50 dark:hover:bg-stone-800/60 transition-colors",
                                                                td { class: "px-6 py-4 font-medium text-gray-900 dark:text-gray-100",
                                                                    Link {
                                                                        to: crate::routes::AuthenticatedRoute::PartyDetail { id: party.id.clone() },
                                                                        class: "hover:underline text-blue-600 dark:text-blue-400 font-semibold flex items-center gap-1",
                                                                        "{party.name} 📖"
                                                                    }
                                                                }
                                                                td { class: "px-6 py-4 text-xs font-semibold",
                                                                    span {
                                                                        class: match party.party_type.as_str() {
                                                                            "CUSTOMER" => "px-2 py-0.5 rounded bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300",
                                                                            "SUPPLIER" => "px-2 py-0.5 rounded bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300",
                                                                            "BAKERY" => "px-2 py-0.5 rounded bg-purple-100 dark:bg-purple-900/50 text-purple-700 dark:text-purple-300",
                                                                            _ => "px-2 py-0.5 rounded bg-emerald-100 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300"
                                                                        },
                                                                        "{party.party_type}"
                                                                    }
                                                                }
                                                                td { class: "px-6 py-4 text-right font-bold text-sm",
                                                                    class: if party.current_balance > 0.0 { "text-emerald-600 dark:text-emerald-400" } else if party.current_balance < 0.0 { "text-rose-600 dark:text-rose-400" } else { "text-gray-500" },
                                                                    "{bal_str}"
                                                                }
                                                                td { class: "px-6 py-4 text-right flex items-center justify-end gap-2",
                                                                    Button {
                                                                        variant: ButtonVariant::Outline,
                                                                        onclick: move |_| {
                                                                            form_id.set(Some(p_edit.id.clone()));
                                                                            form_name.set(p_edit.name.clone());
                                                                            form_type.set(p_edit.party_type.clone());
                                                                            form_balance.set(p_edit.current_balance.to_string());
                                                                            is_sheet_open.set(true);
                                                                        },
                                                                        "{edit_str}"
                                                                    }
                                                                    Button {
                                                                        variant: ButtonVariant::Destructive,
                                                                        onclick: move |_| {
                                                                            delete_id.set(Some(p_del_id.clone()));
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
                        Some(Ok(_)) if !filtered_list.is_empty() => rsx! {
                            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4 mt-2",
                                for party in filtered_list {
                                    {
                                        let bal_str = if party.party_type == "SUPPLIER" {
                                            if party.current_balance < 0.0 {
                                                format!("-₹ {:.2} (Payable)", party.current_balance.abs())
                                            } else if party.current_balance > 0.0 {
                                                format!("+₹ {:.2} (Surplus)", party.current_balance)
                                            } else {
                                                "₹ 0.00".to_string()
                                            }
                                        } else {
                                            if party.current_balance > 0.0 {
                                                format!("₹ {:.2} (Due)", party.current_balance)
                                            } else if party.current_balance < 0.0 {
                                                format!("-₹ {:.2} (Overpaid)", party.current_balance.abs())
                                            } else {
                                                "₹ 0.00".to_string()
                                            }
                                        };
                                        let p_edit = party.clone();
                                        let p_del_id = party.id.clone();
                                        let p_id_link = party.id.clone();
                                        rsx! {
                                            Card { key: "{party.id}",
                                                CardHeader {
                                                    div { class: "flex justify-between items-center",
                                                        Link {
                                                            to: crate::routes::AuthenticatedRoute::PartyDetail { id: p_id_link },
                                                            class: "text-lg font-bold text-gray-900 dark:text-gray-100 hover:text-blue-600 dark:hover:text-blue-400 hover:underline flex items-center gap-1",
                                                            "{party.name} 📖"
                                                        }
                                                        span {
                                                            class: match party.party_type.as_str() {
                                                                "CUSTOMER" => "px-2 py-0.5 rounded text-xs font-semibold bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300",
                                                                "SUPPLIER" => "px-2 py-0.5 rounded text-xs font-semibold bg-amber-100 dark:bg-amber-900/50 text-amber-700 dark:text-amber-300",
                                                                "BAKERY" => "px-2 py-0.5 rounded text-xs font-semibold bg-purple-100 dark:bg-purple-900/50 text-purple-700 dark:text-purple-300",
                                                                _ => "px-2 py-0.5 rounded text-xs font-semibold bg-emerald-100 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300"
                                                            },
                                                            "{party.party_type}"
                                                        }
                                                    }
                                                }
                                                CardContent {
                                                    div { class: "flex justify-between items-center bg-stone-50 dark:bg-stone-800/50 p-3 rounded-lg border border-stone-200/60 dark:border-stone-700/60 text-xs",
                                                        span { class: "text-gray-500 dark:text-gray-400", "Current Balance" }
                                                        span {
                                                            class: if party.current_balance > 0.0 { "text-base font-bold text-emerald-600 dark:text-emerald-400" } else if party.current_balance < 0.0 { "text-base font-bold text-rose-600 dark:text-rose-400" } else { "text-base font-bold text-gray-500" },
                                                            "{bal_str}"
                                                        }
                                                    }
                                                }
                                                CardFooter {
                                                    div { class: "flex justify-end gap-2 w-full pt-1",
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                form_id.set(Some(p_edit.id.clone()));
                                                                form_name.set(p_edit.name.clone());
                                                                form_type.set(p_edit.party_type.clone());
                                                                form_balance.set(p_edit.current_balance.to_string());
                                                                is_sheet_open.set(true);
                                                            },
                                                            "{edit_str}"
                                                        }
                                                        Button {
                                                            variant: ButtonVariant::Destructive,
                                                            onclick: move |_| {
                                                                delete_id.set(Some(p_del_id.clone()));
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
                                icon: "📒",
                                title: "No Parties Found",
                                description: "No party records match your search or selected filter."
                            }
                        },
                        Some(Err(err)) => rsx! { ErrorState { message: err } },
                        None => rsx! { LoadingState {} }
                    }

                    ConfirmDialog {
                        is_open: delete_id().is_some(),
                        is_deleting: is_deleting(),
                        title: "Delete Party".to_string(),
                        description: "Are you sure you want to delete this party? All associated transaction histories will be unlinked.".to_string(),
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
                        SheetTitle { if form_id().is_some() { "Edit Party" } else { "Add New Party" } }
                    }
                    div { class: "flex flex-col gap-4 py-4 px-6 overflow-y-auto max-h-[70vh]",
                        if !form_error().is_empty() {
                            div { class: "p-3 rounded bg-red-50 text-red-600 text-sm border border-red-200", "{form_error}" }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-name", "Party Name" }
                            Input {
                                placeholder: "e.g. John Doe, Fresh Bakery",
                                value: "{form_name}",
                                oninput: move |e: Event<FormData>| form_name.set(e.value())
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-type", "Party Type" }
                            select {
                                class: "flex h-10 w-full rounded-md border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-900 px-3 py-2 text-sm text-stone-900 dark:text-stone-100 shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{form_type}",
                                onchange: move |e: Event<FormData>| form_type.set(e.value()),
                                option { value: "CUSTOMER", "CUSTOMER" }
                                option { value: "SUPPLIER", "SUPPLIER" }
                                option { value: "BAKERY", "BAKERY" }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "party-balance", "Initial Balance (₹)" }
                            Input {
                                r#type: "number",
                                step: "0.01",
                                value: "{form_balance}",
                                oninput: move |e: Event<FormData>| form_balance.set(e.value())
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
                            if form_id().is_some() { "Update Party" } else { "Save Party" }
                        }
                    }
                }
            }
        }
    }
}
