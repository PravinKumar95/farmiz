use dioxus::prelude::*;
use crate::components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::theme_toggle::ThemeToggle;
use crate::i18n::{tr, LanguageSelect};

#[component]
pub fn Settings() -> Element {
    let settings_title = tr("settings");

    rsx! {
        div { class: "flex flex-col h-full w-full min-h-0",
            // FIXED HEADER
            div { class: "shrink-0 p-4 md:p-6 pb-3 border-b border-stone-200/60 dark:border-stone-800 bg-white dark:bg-stone-900 flex flex-col gap-1",
                h1 { class: "text-2xl font-bold tracking-tight text-gray-900 dark:text-gray-100", "⚙️ {settings_title}" }
                p { class: "text-xs text-gray-500 dark:text-gray-400",
                    "Manage your application localization, display preferences, and theme appearance"
                }
            }

            // SCROLLABLE CONTENT
            div { class: "flex-1 overflow-y-auto min-h-0 p-4 md:p-6",
                div { class: "flex flex-col gap-6 w-full max-w-5xl mx-auto pb-20",
                    div { class: "grid gap-6 md:grid-cols-2",
                        // ── Language Preferences Card ──
                        Card {
                            CardHeader {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-xl", "🌐" }
                                    CardTitle { class: "text-lg font-semibold", "Language & Localization" }
                                }
                                CardDescription { "Select your preferred language for the application user interface." }
                            }
                            CardContent { class: "pt-2 space-y-4",
                                div { class: "flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-3 bg-stone-50 dark:bg-stone-800/50 rounded-lg border border-stone-200 dark:border-stone-700/60",
                                    div {
                                        div { class: "text-sm font-medium text-gray-900 dark:text-gray-100", "Display Language" }
                                        div { class: "text-xs text-gray-500 dark:text-gray-400", "Applies to UI labels, navigation, and tables" }
                                    }
                                    LanguageSelect {}
                                }
                            }
                        }

                        // ── Appearance & Theme Card ──
                        Card {
                            CardHeader {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-xl", "🎨" }
                                    CardTitle { class: "text-lg font-semibold", "Appearance & Theme" }
                                }
                                CardDescription { "Switch between light and dark color themes." }
                            }
                            CardContent { class: "pt-2 space-y-4",
                                div { class: "flex items-center justify-between gap-3 p-3 bg-stone-50 dark:bg-stone-800/50 rounded-lg border border-stone-200 dark:border-stone-700/60",
                                    div {
                                        div { class: "text-sm font-medium text-gray-900 dark:text-gray-100", "Interface Mode" }
                                        div { class: "text-xs text-gray-500 dark:text-gray-400", "Toggle Light or Dark theme" }
                                    }
                                    ThemeToggle {}
                                }
                            }
                        }
                    }

                    // ── System Information Card ──
                    Card {
                        CardHeader {
                            div { class: "flex items-center gap-2",
                                span { class: "text-xl", "ℹ️" }
                                CardTitle { class: "text-lg font-semibold", "About Farmiz Poultry App" }
                            }
                            CardDescription { "System metadata and software version." }
                        }
                        CardContent { class: "grid gap-3 sm:grid-cols-3 text-sm pt-2",
                            div { class: "p-3 bg-stone-50 dark:bg-stone-800/50 rounded-lg border border-stone-200 dark:border-stone-700/60",
                                div { class: "text-xs text-gray-500 dark:text-gray-400", "Version" }
                                div { class: "font-semibold text-gray-900 dark:text-gray-100 mt-0.5", "v0.1.0" }
                            }
                            div { class: "p-3 bg-stone-50 dark:bg-stone-800/50 rounded-lg border border-stone-200 dark:border-stone-700/60",
                                div { class: "text-xs text-gray-500 dark:text-gray-400", "Engine" }
                                div { class: "font-semibold text-gray-900 dark:text-gray-100 mt-0.5", "Dioxus (Rust WASM)" }
                            }
                            div { class: "p-3 bg-stone-50 dark:bg-stone-800/50 rounded-lg border border-stone-200 dark:border-stone-700/60",
                                div { class: "text-xs text-gray-500 dark:text-gray-400", "Status" }
                                div { class: "font-semibold text-green-600 dark:text-green-400 mt-0.5", "Online" }
                            }
                        }
                    }
                }
            }
        }
    }
}
