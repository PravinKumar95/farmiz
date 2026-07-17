use dioxus::prelude::*;

use crate::routes::PublicRoute;

#[component]
pub fn PublicLayout() -> Element {
    rsx! {
        div { class: "max-w-md mx-auto w-full p-4 flex flex-col justify-center",
            Outlet::<PublicRoute> {}
        }
    }
}
