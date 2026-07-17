use dioxus::logger::tracing;
use dioxus::prelude::*;
use tracing::Level;

mod components;
mod layouts;
mod routes;
mod screens;

use crate::routes::PublicRoute;


const FAVICON: Asset = asset!("/assets/favicon.ico");

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        dioxus_sdk::storage::set_dir!();
    }
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[derive(Clone, Copy)]
pub struct LoginAction(pub Callback<crate::screens::login::LoginInfo>);

#[derive(Clone, Copy)]
pub struct LogoutAction(pub Callback<()>);

#[component]
fn App() -> Element {
    let mut auth_token = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_token".to_string(),
        || None::<String>,
    );
    let mut auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_email".to_string(),
        || None::<String>,
    );

    use_context_provider(|| {
        LoginAction(Callback::new(move |info: crate::screens::login::LoginInfo| {
            auth_token.set(Some(info.token));
            auth_email.set(Some(info.email));
        }))
    });

    use_context_provider(|| {
        LogoutAction(Callback::new(move |()| {
            auth_token.set(None);
            auth_email.set(None);
        }))
    });

    rsx! {
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0, viewport-fit=cover",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }
        document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
        div { class: "pt-25 h-screen flex flex-col items-center dark:bg-stone-900",
            if auth_token().is_some() {
                // ── Authenticated routes ──
                Router::<crate::routes::AuthenticatedRoute> {}
            } else {
                Router::<PublicRoute> {}
            }
        }
    }
}
