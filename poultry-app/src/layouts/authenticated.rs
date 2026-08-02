use dioxus::prelude::*;
use dioxus_router::components::Outlet;

use crate::components::button::{Button, ButtonVariant};
use crate::components::sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider,
    SidebarTrigger,
};
use crate::routes::AuthenticatedRoute;
use crate::i18n::tr;
use crate::components::theme_toggle::ThemeToggle;

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
    let signout_lbl = tr("sign-out");

    let logout_action = use_context::<crate::LogoutAction>();
    let on_signout = move |_| {
        logout_action.0.call(());
    };

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
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Settings {},
                                        SidebarMenuButton {
                                            span { "⚙️ {settings_lbl}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                SidebarFooter {
                    div { class: "p-4 flex flex-col gap-3 border-t border-stone-200/60 dark:border-stone-800",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: on_signout,
                            class: "w-full flex items-center justify-center gap-2 text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 border-red-200 dark:border-red-900/50 hover:bg-red-50 dark:hover:bg-red-950/30",
                            "🚪 {signout_lbl}"
                        }
                        div { class: "text-xs text-gray-500 dark:text-gray-400 text-center", "Farmiz v0.1.0" }
                    }
                }
            }
            SidebarInset {
                header { class: "flex h-14 items-center justify-between gap-4 border-b bg-white dark:bg-stone-900 px-4 lg:h-[60px] shrink-0",
                    SidebarTrigger {}
                    ThemeToggle {}
                }
                div { class: "flex-1 overflow-hidden flex flex-col min-h-0 text-gray-900 dark:text-gray-100",
                    Outlet::<AuthenticatedRoute> {}
                }
            }
        }
    }
}
