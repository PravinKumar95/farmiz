use crate::components::button::Button;
use crate::components::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::toast::{use_toast, ToastOptions};
use dioxus::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct SignInPayload {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct SignUpPayload {
    name: String,
    email: String,
    password: String,
}

#[derive(Clone)]
pub struct LoginInfo {
    pub token: String,
    pub email: String,
    pub session_cookies: Option<Vec<String>>,
}

#[component]
pub fn SignIn() -> Element {
    let login_action = use_context::<crate::LoginAction>();
    let on_login = login_action.0;
    let toast_api = use_toast();

    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut error_msg = use_signal(|| "".to_string());
    let mut is_loading = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        evt.stop_propagation();
        is_loading.set(true);
        error_msg.set("".to_string());

        let login_email = email().clone();

        spawn(async move {
            let payload = SignInPayload {
                email: email(),
                password: password(),
            };

            let backend_url = option_env!("BACKEND_URL")
                .unwrap_or("http://127.0.0.1:9000/lambda-url/poultry-backend");
            println!("Attempting sign in with URL: {}", backend_url);

            let client = reqwest::Client::new();
            println!("Client created, sending request...");
            let res = client
                .post(format!("{}/api/auth/signin", backend_url))
                .json(&payload)
                .send()
                .await;

            println!("Request completed with result: {:?}", res.is_ok());

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            let token_str = json.get("token").and_then(|t| t.as_str())
                                .or_else(|| json.get("session").and_then(|s| s.get("access_token")).and_then(|t| t.as_str()));
                            
                            let session_cookies = json.get("session_cookies").and_then(|c| {
                                c.as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                            });
                            
                            if let Some(token) = token_str {
                                println!("Login successful!");
                                toast_api.success("Success".to_string(), ToastOptions::new().description("Signed in successfully."));
                                on_login.call(LoginInfo {
                                    token: token.to_string(),
                                    email: login_email,
                                    session_cookies,
                                });
                                is_loading.set(false);
                                return;
                            }
                        }
                        let err = "Signed in, but no token returned.".to_string();
                        error_msg.set(err.clone());
                        toast_api.error("Error".to_string(), ToastOptions::new().description(err));
                    } else {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        println!("Server error: {} - {}", status, body);
                        let msg = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body)
                        {
                            json.get("message")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or(format!("Error: {}", status))
                        } else {
                            format!("Error: {}", status)
                        };
                        error_msg.set(msg.clone());
                        toast_api.error("Error".to_string(), ToastOptions::new().description(msg));
                    }
                }
                Err(e) => {
                    println!("Network error: {:?}", e);
                    let err = format!("Request failed: {}", e);
                    error_msg.set(err.clone());
                    toast_api.error("Error".to_string(), ToastOptions::new().description(err));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div { class: "w-full space-y-6",
            div { class: "space-y-2 text-center sm:text-left",
                h1 { class: "text-3xl font-bold tracking-tight text-stone-900 dark:text-white", "Welcome back" }
                p { class: "text-sm text-stone-500 dark:text-stone-400", "Sign in to your account" }
            }

            form { onsubmit: handle_submit, class: "space-y-4",
                div { class: "space-y-2",
                    Label { html_for: "signin_email", "Email" }
                    Input {
                        id: "signin_email",
                        r#type: "email",
                        placeholder: "you@example.com",
                        value: "{email}",
                        oninput: move |e: Event<FormData>| email.set(e.value()),
                        required: true,
                        disabled: is_loading(),
                    }
                }

                div { class: "space-y-2",
                    div { class: "flex items-center justify-between",
                        Label { html_for: "signin_password", "Password" }
                        a { href: "#", class: "text-xs text-stone-500 hover:text-stone-700 dark:text-stone-400 dark:hover:text-stone-200 transition-colors", "Forgot password?" }
                    }
                    Input {
                        id: "signin_password",
                        r#type: "password",
                        placeholder: "••••••••",
                        value: "{password}",
                        oninput: move |e: Event<FormData>| password.set(e.value()),
                        required: true,
                        disabled: is_loading(),
                    }
                }

                button {
                    r#type: "submit",
                    disabled: is_loading(),
                    class: "w-full h-11 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white font-semibold rounded-lg shadow-sm transition-all duration-150 flex items-center justify-center cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed mt-6",
                    if is_loading() {
                        "Signing in..."
                    } else {
                        "Sign in"
                    }
                }
            }

            div { class: "text-center text-sm text-stone-600 dark:text-stone-400 pt-2",
                "Don't have an account? "
                Link {
                    to: crate::routes::PublicRoute::SignUp,
                    class: "font-semibold text-emerald-600 hover:text-emerald-500 dark:text-emerald-400 dark:hover:text-emerald-300 underline underline-offset-4 transition-colors",
                    "Sign up"
                }
            }

            if !error_msg().is_empty() {
                div { class: "p-3.5 bg-red-500/10 border border-red-500/20 text-sm rounded-lg text-red-600 dark:text-red-400 font-medium",
                    "{error_msg}"
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum SignUpState {
    Form,
    Loading,
    VerifyOtp { email_address: String },
    Verifying,
    Verified,
}

#[derive(Clone, PartialEq)]
enum MessageKind {
    None,
    Error(String),
}

fn extract_error_message(err_body: &str) -> String {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(err_body) {
        if let Some(m) = json.get("message").and_then(|m| m.as_str()) {
            return m.to_string();
        }
        if let Some(err_val) = json.get("error") {
            if let Some(e) = err_val.as_str() {
                return e.to_string();
            }
            if let Some(m) = err_val.get("message").and_then(|m| m.as_str()) {
                return m.to_string();
            }
        }
    }
    if err_body.is_empty() {
        "An unexpected error occurred".to_string()
    } else {
        err_body.to_string()
    }
}

#[derive(Serialize)]
struct VerifyEmailPayload {
    email: String,
    otp: String,
}

#[component]
pub fn SignUp() -> Element {
    let toast_api = use_toast();
    let mut name = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut otp_code = use_signal(|| "".to_string());
    let mut state = use_signal(|| SignUpState::Form);
    let mut feedback = use_signal(|| MessageKind::None);

    let handle_signup = move |evt: Event<FormData>| {
        evt.prevent_default();
        evt.stop_propagation();
        state.set(SignUpState::Loading);
        feedback.set(MessageKind::None);

        let submitted_email = email().clone();

        spawn(async move {
            let payload = SignUpPayload {
                name: name(),
                email: email(),
                password: password(),
            };

            let backend_url = option_env!("BACKEND_URL")
                .unwrap_or("http://127.0.0.1:9000/lambda-url/poultry-backend");
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/api/auth/signup", backend_url))
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        state.set(SignUpState::VerifyOtp {
                            email_address: submitted_email,
                        });
                        toast_api.success("Success".to_string(), ToastOptions::new().description("Verification code sent to your email."));
                    } else {
                        let err_body = resp.text().await.unwrap_or_default();
                        let msg = extract_error_message(&err_body);
                        feedback.set(MessageKind::Error(msg.clone()));
                        toast_api.error("Error".to_string(), ToastOptions::new().description(msg));
                        state.set(SignUpState::Form);
                    }
                }
                Err(e) => {
                    let err = format!("Request failed: {}", e);
                    feedback.set(MessageKind::Error(err.clone()));
                    toast_api.error("Error".to_string(), ToastOptions::new().description(err));
                    state.set(SignUpState::Form);
                }
            }
        });
    };

    let handle_verify = move |evt: Event<FormData>| {
        evt.prevent_default();
        evt.stop_propagation();
        feedback.set(MessageKind::None);

        let current_email = email().clone();
        let code = otp_code().clone();

        state.set(SignUpState::Verifying);

        spawn(async move {
            let payload = VerifyEmailPayload {
                email: current_email,
                otp: code,
            };

            let backend_url = option_env!("BACKEND_URL")
                .unwrap_or("http://127.0.0.1:9000/lambda-url/poultry-backend");
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/api/auth/verify-email", backend_url))
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        state.set(SignUpState::Verified);
                        toast_api.success("Success".to_string(), ToastOptions::new().description("Email verified successfully! You can now sign in."));
                    } else {
                        let err_body = resp.text().await.unwrap_or_default();
                        let msg = extract_error_message(&err_body);
                        feedback.set(MessageKind::Error(msg.clone()));
                        toast_api.error("Error".to_string(), ToastOptions::new().description(msg));
                        // Go back to the OTP input so they can retry
                        state.set(SignUpState::VerifyOtp {
                            email_address: email(),
                        });
                    }
                }
                Err(e) => {
                    let err = format!("Request failed: {}", e);
                    feedback.set(MessageKind::Error(err.clone()));
                    toast_api.error("Error".to_string(), ToastOptions::new().description(err));
                    state.set(SignUpState::VerifyOtp {
                        email_address: email(),
                    });
                }
            }
        });
    };

    match state() {
        // ── OTP Verification Screen ──
        SignUpState::VerifyOtp { email_address } => {
            let display_email = email_address;
            let is_verifying = false;
            rsx! {
                Card {
                    CardHeader {
                        div { class: "text-4xl mb-2 text-center", "✉️" }
                        CardTitle { class: "text-2xl font-bold text-center text-gray-900 dark:text-gray-100", "Check Your Email" }
                        CardDescription { class: "text-center text-gray-600 dark:text-gray-400 mt-1",
                            "We sent a verification code to:"
                            div { class: "font-semibold text-gray-900 dark:text-gray-100 mt-1 break-all", "{display_email}" }
                        }
                    }
                    CardContent {
                        form {
                            onsubmit: handle_verify,
                            class: "flex flex-col gap-4 mt-2",
                            Input {
                                r#type: "text",
                                placeholder: "Enter 6-digit code",
                                value: "{otp_code}",
                                oninput: move |e: Event<FormData>| otp_code.set(e.value()),
                                class: "text-center text-xl tracking-widest font-mono text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md p-3 focus:ring-2 focus:ring-blue-500 outline-none",
                                maxlength: "6",
                                required: true,
                                disabled: is_verifying,
                                autocomplete: "one-time-code",
                            }
                            Button { r#type: "submit", class: "w-full", disabled: is_verifying, "Verify Email" }
                        }

                        match feedback() {
                            MessageKind::Error(msg) => rsx! {
                                div { class: "mt-3 p-3 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-800 text-sm rounded text-red-700 dark:text-red-300 text-left",
                                    "{msg}"
                                }
                            },
                            MessageKind::None => rsx! {},
                        }

                        p { class: "text-xs text-center text-gray-500 dark:text-gray-400 mt-4",
                            "Didn't receive the code? Check your spam folder or try signing up again."
                        }
                    }
                }
            }
        }

        SignUpState::Verifying => {
            let display_email = email();
            rsx! {
                Card {
                    CardHeader {
                        div { class: "text-4xl mb-2 text-center", "✉️" }
                        CardTitle { class: "text-2xl font-bold text-center text-gray-900 dark:text-gray-100", "Verifying Code..." }
                        CardDescription { class: "text-center text-gray-600 dark:text-gray-400 mt-1",
                            "Verifying code sent to:"
                            div { class: "font-semibold text-gray-900 dark:text-gray-100 mt-1 break-all", "{display_email}" }
                        }
                    }
                    CardContent {
                        form {
                            onsubmit: handle_verify,
                            class: "flex flex-col gap-4 mt-2",
                            Input {
                                r#type: "text",
                                placeholder: "Enter 6-digit code",
                                value: "{otp_code}",
                                oninput: move |e: Event<FormData>| otp_code.set(e.value()),
                                class: "text-center text-xl tracking-widest font-mono text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md p-3 focus:ring-2 focus:ring-blue-500 outline-none",
                                maxlength: "6",
                                required: true,
                                disabled: true,
                                autocomplete: "one-time-code",
                            }
                            Button { r#type: "submit", class: "w-full", disabled: true, loading: true, "Verifying..." }
                        }

                        match feedback() {
                            MessageKind::Error(msg) => rsx! {
                                div { class: "mt-3 p-3 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-800 text-sm rounded text-red-700 dark:text-red-300 text-left",
                                    "{msg}"
                                }
                            },
                            MessageKind::None => rsx! {},
                        }

                        p { class: "text-xs text-center text-gray-500 dark:text-gray-400 mt-4",
                            "Didn't receive the code? Check your spam folder or try signing up again."
                        }
                    }
                }
            }
        }

        // ── Verified Success Screen ──
        SignUpState::Verified => {
            rsx! {
                Card {
                    CardHeader {
                        div { class: "text-4xl mb-2 text-center", "✅" }
                        CardTitle { class: "text-2xl font-bold text-center text-green-700 dark:text-green-400", "Email Verified!" }
                        CardDescription { class: "text-center text-gray-600 dark:text-gray-400 mt-2",
                            "Your account is ready. You can now sign in with your credentials."
                        }
                    }
                    CardContent { class: "pt-2",
                        Link {
                            to: crate::routes::PublicRoute::SignIn {},
                            class: "w-full block text-center bg-blue-600 text-white font-medium py-2.5 px-4 rounded-md hover:bg-blue-700 transition-colors dark:bg-blue-500 dark:hover:bg-blue-600",
                            "Proceed to Sign In"
                        }
                    }
                }
            }
        }

        // ── Sign Up Form (default + loading) ──
        _ => {
            let is_loading = state() == SignUpState::Loading;
            rsx! {
                div { class: "w-full space-y-6",
                    div { class: "space-y-2 text-center sm:text-left",
                        h1 { class: "text-3xl font-bold tracking-tight text-stone-900 dark:text-white", "Create an account" }
                        p { class: "text-sm text-stone-500 dark:text-stone-400", "Enter your details below to get started" }
                    }

                    form { onsubmit: handle_signup, class: "space-y-4",
                        div { class: "space-y-2",
                            Label { html_for: "signup_name", "Full Name" }
                            Input {
                                id: "signup_name",
                                r#type: "text",
                                placeholder: "Jane Doe",
                                value: "{name}",
                                oninput: move |e: Event<FormData>| name.set(e.value()),
                                required: true,
                                disabled: is_loading,
                            }
                        }

                        div { class: "space-y-2",
                            Label { html_for: "signup_email", "Email" }
                            Input {
                                id: "signup_email",
                                r#type: "email",
                                placeholder: "you@example.com",
                                value: "{email}",
                                oninput: move |e: Event<FormData>| email.set(e.value()),
                                required: true,
                                disabled: is_loading,
                            }
                        }

                        div { class: "space-y-2",
                            Label { html_for: "signup_password", "Password" }
                            Input {
                                id: "signup_password",
                                r#type: "password",
                                placeholder: "••••••••",
                                value: "{password}",
                                oninput: move |e: Event<FormData>| password.set(e.value()),
                                required: true,
                                disabled: is_loading,
                            }
                        }

                        match feedback() {
                            MessageKind::Error(msg) => rsx! {
                                div { class: "p-3.5 bg-red-500/10 border border-red-500/20 text-sm rounded-lg text-red-600 dark:text-red-400 font-medium", "{msg}" }
                            },
                            MessageKind::None => rsx! {},
                        }

                        button {
                            r#type: "submit",
                            disabled: is_loading,
                            class: "w-full h-11 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white font-semibold rounded-lg shadow-sm transition-all duration-150 flex items-center justify-center cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed mt-6",
                            if is_loading {
                                "Creating account..."
                            } else {
                                "Sign up"
                            }
                        }
                    }

                    div { class: "text-center text-sm text-stone-600 dark:text-stone-400 pt-2",
                        "Already have an account? "
                        Link {
                            to: crate::routes::PublicRoute::SignIn {},
                            class: "font-semibold text-emerald-600 hover:text-emerald-500 dark:text-emerald-400 dark:hover:text-emerald-300 underline underline-offset-4 transition-colors",
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
