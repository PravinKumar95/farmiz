use dioxus::logger::tracing;
use dioxus::prelude::*;
use tracing::Level;

mod components;
mod i18n;
mod layouts;
mod models;
mod routes;
mod screens;
mod services;

use crate::routes::PublicRoute;


const LOGO: Asset = asset!("/assets/logo.png");

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
    let _i18n = i18n::use_app_i18n();
    let mut auth_token = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_token".to_string(),
        || None::<String>,
    );
    let mut auth_email = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "auth_email".to_string(),
        || None::<String>,
    );

    let mut session_cookies = dioxus_sdk::storage::use_storage::<dioxus_sdk::storage::LocalStorage, _>(
        "session_cookies".to_string(),
        || None::<Vec<String>>,
    );

    use_context_provider(|| {
        LoginAction(Callback::new(move |info: crate::screens::login::LoginInfo| {
            auth_token.set(Some(info.token));
            auth_email.set(Some(info.email));
            session_cookies.set(info.session_cookies);
        }))
    });

    use_context_provider(|| {
        LogoutAction(Callback::new(move |()| {
            auth_token.set(None);
            auth_email.set(None);
            session_cookies.set(None);
        }))
    });

    rsx! {
        crate::components::toast::ToastProvider {
            document::Meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1.0, viewport-fit=cover",
            }
            document::Link { rel: "icon", href: LOGO }
            document::Link { rel: "apple-touch-icon", href: LOGO }

            document::Stylesheet {
                // Urls are relative to your Cargo.toml file
                href: asset!("/assets/tailwind.css"),
            }
            document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
            if auth_token().is_some() {
                // ── Authenticated routes ──
                div { class: "w-full h-[100dvh] dark:bg-stone-900",
                    Router::<crate::routes::AuthenticatedRoute> {}
                }
            } else {
                div { class: "min-h-[100dvh] w-full bg-stone-50 dark:bg-stone-950 text-stone-900 dark:text-stone-100 flex flex-col justify-between selection:bg-emerald-500 selection:text-white transition-colors duration-200",
                    header { class: "w-full max-w-7xl mx-auto px-6 py-6 flex items-center justify-between",
                        div { class: "flex items-center gap-2.5 font-bold text-xl tracking-tight text-stone-900 dark:text-white",
                            crate::components::logo::Logo { class: "h-7 w-7" }
                            span { "farmiz" }
                        }
                        crate::components::theme_toggle::ThemeToggle {}
                    }
                    main { class: "flex-1 flex items-center justify-center p-4 sm:p-6",
                        div { class: "w-full max-w-md",
                            Router::<PublicRoute> {}
                        }
                    }
                    footer { class: "w-full max-w-7xl mx-auto px-6 py-6 text-center text-xs text-stone-500 dark:text-stone-500",
                        "By continuing, you agree to Farmiz Terms of Service and Privacy Policy."
                    }
                }
            }
        }
    }
}
