use dioxus::prelude::*;
use dioxus_router::components::Outlet;

use crate::components::sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent, SidebarGroupLabel,
    SidebarHeader, SidebarInset, SidebarMenu, SidebarMenuButton, SidebarMenuItem, SidebarProvider,
    SidebarTrigger,
};
use crate::routes::AuthenticatedRoute;

#[component]
pub fn AuthenticatedLayout() -> Element {
    rsx! {
        SidebarProvider {
            Sidebar {
                SidebarHeader {
                    div { class: " p-4 font-bold text-lg text-primary", "🐔 Farmiz App" }
                }
                SidebarContent {
                    SidebarGroup {
                        SidebarGroupLabel { "Navigation" }
                        SidebarGroupContent {
                            SidebarMenu {
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Dashboard {},
                                        SidebarMenuButton {
                                            span { "🏠 Dashboard" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Sales {},
                                        SidebarMenuButton {
                                            span { "🥚 Sales" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Purchases {},
                                        SidebarMenuButton {
                                            span { "🛒 Purchases" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Feed {},
                                        SidebarMenuButton {
                                            span { "🌾 Feed Mill" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Labor {},
                                        SidebarMenuButton {
                                            span { "👥 Labor" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Parties {},
                                        SidebarMenuButton {
                                            span { "📒 Ledger" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    dioxus_router::components::Link { to: AuthenticatedRoute::Employees {},
                                        SidebarMenuButton {
                                            span { "👷 Employees" }
                                        }
                                    }
                                }
                                SidebarMenuItem {
                                    SidebarMenuButton {
                                        span { "⚙️ Settings" }
                                    }
                                }
                            }
                        }
                    }
                }
                SidebarFooter {
                    div { class: "p-4 text-xs text-muted-foreground", "Farmiz v0.1.0" }
                }
            }
            SidebarInset {
                header { class: "flex h-14 items-center gap-4 border-b bg-background px-4 lg:h-[60px]",
                    SidebarTrigger {}
                }
                main { class: "flex flex-1 flex-col gap-4 p-4 md:gap-8 md:p-6", Outlet::<AuthenticatedRoute> {} }
            }
        }
    }
}
