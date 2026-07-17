use dioxus::prelude::*;

use crate::routes::PublicRoute;

#[component]
pub fn PublicLayout() -> Element {
    rsx! {
        Outlet::<PublicRoute> {}
    }
}
