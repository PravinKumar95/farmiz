use dioxus::prelude::*;
use dioxus_router::components::Outlet;

use crate::components::sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider,
    SidebarTrigger,
};
use crate::routes::AuthenticatedRoute;
use crate::i18n::{tr, LanguageSelect};

#[component]
pub fn AuthenticatedLayout() -> Element {
    let title = tr("app-title");
    let dashboard_lbl = tr("dashboard");
    let production_lbl = tr("production");
    let sales_lbl = tr("sales");
    let purchases_lbl = tr("purchases");
    let feed_lbl = tr("feed-mill");
    let labor_lbl = tr("labor");
    let ledger_lbl = tr("ledger");
    let employees_lbl = tr("employees");
    let settings_lbl = tr("settings");

    rsx! {
        SidebarProvider {
            Sidebar {
                SidebarHeader {
                    div { class: " p-4 font-bold text-lg text-blue-600 dark:text-blue-400", "🐔 {title}" }
                }
                SidebarContent {
                    SidebarGroup {
                        SidebarGroupLabel { "Navigation" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Dashboard {},
                                        SidebarMenuButton {
                                            span { "🏠 {dashboard_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Production {},
                                        SidebarMenuButton {
                                            span { "🥚 {production_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Sales {},
                                        SidebarMenuButton {
                                            span { "🥚 {sales_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Purchases {},
                                        SidebarMenuButton {
                                            span { "🛒 {purchases_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Feed {},
                                        SidebarMenuButton {
                                            span { "🌾 {feed_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Labor {},
                                        SidebarMenuButton {
                                            span { "👥 {labor_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Parties {},
                                        SidebarMenuButton {
                                            span { "📒 {ledger_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Employees {},
                                        SidebarMenuButton {
                                            span { "👷 {employees_lbl}" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        span { "⚙️ {settings_lbl}" }
                                    }
                                }
                            }
                        }
                    }
                }
                SidebarFooter {
                    div { class: "p-4 text-xs text-gray-500 dark:text-gray-400", "Farmiz v0.1.0" }
                }
            }
            SidebarInset {
                header { class: "flex h-14 items-center justify-between gap-4 border-b bg-white dark:bg-stone-900 px-4 lg:h-[60px] shrink-0",
                    SidebarTrigger {}
                    LanguageSelect {}
                }
                div { class: "flex-1 overflow-hidden flex flex-col min-h-0 text-gray-900 dark:text-gray-100",
                    Outlet::<AuthenticatedRoute> {}
                }
            }
        }
    }
}
