use dioxus::prelude::*;
use crate::models::Party;
use crate::services::use_parties;

#[derive(Props, Clone, PartialEq)]
pub struct PartySelectProps {
    pub value: String,
    pub filter_type: Option<String>,
    pub onchange: EventHandler<(String, String)>, // (party_id, party_name)
    #[props(default = false)]
    pub required: bool,
    #[props(default = "Select a party...".to_string())]
    pub placeholder: String,
}

#[component]
pub fn PartySelect(props: PartySelectProps) -> Element {
    let parties_res = use_parties();
    let parties = parties_res.cloned().and_then(|r| r.ok()).unwrap_or_default();

    let filtered_parties: Vec<Party> = if let Some(ref f_type) = props.filter_type {
        let matched: Vec<Party> = parties
            .iter()
            .filter(|p| p.party_type.to_uppercase() == f_type.to_uppercase())
            .cloned()
            .collect();
        if matched.is_empty() {
            // Fall back to showing all parties if no parties match the specific filter
            parties
        } else {
            matched
        }
    } else {
        parties
    };

    let parties_for_onchange = filtered_parties.clone();

    rsx! {
        select {
            class: "flex h-10 w-full rounded-md border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-900 px-3 py-2 text-sm text-stone-900 dark:text-stone-100 shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500",
            value: "{props.value}",
            required: props.required,
            onchange: move |evt: Event<FormData>| {
                let selected_val = evt.value();
                if let Some(matched) = parties_for_onchange.iter().find(|p| p.id == selected_val || p.name == selected_val) {
                    props.onchange.call((matched.id.clone(), matched.name.clone()));
                } else {
                    props.onchange.call((String::new(), selected_val));
                }
            },
            option { value: "", disabled: props.required, "{props.placeholder}" }
            for p in filtered_parties {
                option {
                    key: "{p.id}",
                    value: "{p.id}",
                    selected: props.value == p.id || props.value == p.name,
                    "{p.name} ({p.party_type})"
                }
            }
        }
    }
}
