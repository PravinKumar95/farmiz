use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use unic_langid::{langid, LanguageIdentifier};

static EN_US: &str = include_str!("../locales/en-US.ftl");
static TA_IN: &str = include_str!("../locales/ta-IN.ftl");
static HI_IN: &str = include_str!("../locales/hi-IN.ftl");
static TE_IN: &str = include_str!("../locales/te-IN.ftl");
static KN_IN: &str = include_str!("../locales/kn-IN.ftl");
static ML_IN: &str = include_str!("../locales/ml-IN.ftl");

#[derive(Clone, PartialEq)]
pub struct LanguageOption {
    pub id: &'static str,
    pub name: &'static str,
    pub langid: LanguageIdentifier,
}

pub fn available_languages() -> Vec<LanguageOption> {
    vec![
        LanguageOption { id: "en-US", name: "English", langid: langid!("en-US") },
        LanguageOption { id: "ta-IN", name: "தமிழ் (Tamil)", langid: langid!("ta-IN") },
        LanguageOption { id: "hi-IN", name: "हिंदी (Hindi)", langid: langid!("hi-IN") },
        LanguageOption { id: "te-IN", name: "తెలుగు (Telugu)", langid: langid!("te-IN") },
        LanguageOption { id: "kn-IN", name: "ಕನ್ನಡ (Kannada)", langid: langid!("kn-IN") },
        LanguageOption { id: "ml-IN", name: "മലയാളം (Malayalam)", langid: langid!("ml-IN") },
    ]
}

pub fn use_app_i18n() -> I18n {
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_fallback(langid!("en-US"))
            .with_locale((langid!("en-US"), EN_US))
            .with_locale((langid!("ta-IN"), TA_IN))
            .with_locale((langid!("hi-IN"), HI_IN))
            .with_locale((langid!("te-IN"), TE_IN))
            .with_locale((langid!("kn-IN"), KN_IN))
            .with_locale((langid!("ml-IN"), ML_IN))
    })
}

pub fn tr(key: &str) -> String {
    let i18n_ctx = i18n();
    i18n_ctx.try_translate(key).unwrap_or_else(|_| key.to_string())
}

#[component]
pub fn LanguageSelect() -> Element {
    let mut i18n_ctx = i18n();
    let current_lang = i18n_ctx.language().to_string();

    rsx! {
        div { class: "flex items-center gap-2",
            span { class: "text-xs text-gray-500 dark:text-gray-400 font-medium hidden sm:inline", "🌐 Language:" }
            select {
                class: "bg-gray-50 dark:bg-stone-800 border border-gray-300 dark:border-stone-700 text-gray-900 dark:text-gray-100 text-xs rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-1.5 cursor-pointer font-medium",
                value: "{current_lang}",
                onchange: move |evt: Event<FormData>| {
                    let val = evt.value();
                    if let Ok(lang) = val.parse::<LanguageIdentifier>() {
                        i18n_ctx.set_language(lang);
                    }
                },
                for lang in available_languages() {
                    option {
                        value: "{lang.id}",
                        selected: current_lang.starts_with(lang.id) || current_lang == lang.id,
                        "{lang.name}"
                    }
                }
            }
        }
    }
}
