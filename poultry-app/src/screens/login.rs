use crate::components::button::Button;
use crate::components::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::input::Input;
use crate::components::label::Label;
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
                                on_login.call(LoginInfo {
                                    token: token.to_string(),
                                    email: login_email,
                                    session_cookies,
                                });
                                is_loading.set(false);
                                return;
                            }
                        }
                        error_msg.set("Signed in, but no token returned.".into());
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
                        error_msg.set(msg);
                    }
                }
                Err(e) => {
                    println!("Network error: {:?}", e);
                    error_msg.set(format!("Request failed: {}", e));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { class: "text-2xl font-bold text-center", "Welcome back" }
                CardDescription { "Enter your email below to login to your farmiz account" }
            }
            CardContent {
                form { onsubmit: handle_submit,
                    div { class: "flex flex-col gap-4",
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "signin_email", "Email" }
                            Input {
                                id: "signin_email",
                                r#type: "email",
                                placeholder: "m@example.com",
                                value: "{email}",
                                oninput: move |e: Event<FormData>| email.set(e.value()),
                                required: true,
                                disabled: is_loading(),
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "signin_password", "Password" }
                            Input {
                                id: "signin_password",
                                r#type: "password",
                                value: "{password}",
                                oninput: move |e: Event<FormData>| password.set(e.value()),
                                required: true,
                                disabled: is_loading(),
                            }
                        }
                    }
                    CardFooter { class: "pt-4 flex flex-col gap-3",
                        Button {
                            r#type: "submit",
                            class: "w-full",
                            disabled: is_loading(),
                            if is_loading() {
                                "Signing in..."
                            } else {
                                "Sign In"
                            }
                        }
                        div { class: "text-sm text-center text-gray-500 w-full mt-2",
                            "Don't have an account?"
                            Link {
                                to: crate::routes::PublicRoute::SignUp,
                                class: "ml-1 text-blue-600 hover:underline dark:text-blue-400 font-medium",
                                "Sign up"
                            }
                        }
                    }
                }
                if !error_msg().is_empty() {
                    div { class: "p-3 bg-red-50 border border-red-200 text-sm rounded text-red-700",
                        "{error_msg}"
                    }
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

#[derive(Serialize)]
struct VerifyEmailPayload {
    email: String,
    otp: String,
}

#[component]
pub fn SignUp() -> Element {
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
                    } else {
                        let err_body = resp.text().await.unwrap_or_default();
                        let msg = if let Ok(json) =
                            serde_json::from_str::<serde_json::Value>(&err_body)
                        {
                            json.get("message")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                                .or_else(|| {
                                    json.get("error")
                                        .and_then(|e| e.as_str())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or(err_body)
                        } else {
                            err_body
                        };
                        feedback.set(MessageKind::Error(msg));
                        state.set(SignUpState::Form);
                    }
                }
                Err(e) => {
                    feedback.set(MessageKind::Error(format!("Request failed: {}", e)));
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
                    } else {
                        let err_body = resp.text().await.unwrap_or_default();
                        let msg = if let Ok(json) =
                            serde_json::from_str::<serde_json::Value>(&err_body)
                        {
                            json.get("message")
                                .and_then(|m| m.as_str())
                                .map(|s| s.to_string())
                                .or_else(|| {
                                    json.get("error")
                                        .and_then(|e| e.as_str())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or(err_body)
                        } else {
                            err_body
                        };
                        feedback.set(MessageKind::Error(msg));
                        // Go back to the OTP input so they can retry
                        state.set(SignUpState::VerifyOtp {
                            email_address: email(),
                        });
                    }
                }
                Err(e) => {
                    feedback.set(MessageKind::Error(format!("Request failed: {}", e)));
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
                div { class: "flex flex-col gap-4 p-4 max-w-sm mx-auto text-center",
                    div { class: "text-4xl mb-2", "✉️" }
                    h2 { class: "text-2xl font-bold", "Check Your Email" }
                    p { class: "text-gray-600 mt-1", "We sent a verification code to:" }
                    p { class: "font-semibold text-black mt-1", "{display_email}" }

                    form {
                        onsubmit: handle_verify,
                        class: "flex flex-col gap-3 mt-4",
                        Input {
                            r#type: "text",
                            placeholder: "Enter 6-digit code",
                            value: "{otp_code}",
                            oninput: move |e: Event<FormData>| otp_code.set(e.value()),
                            class: "text-center text-xl tracking-widest font-mono",
                            maxlength: "6",
                            required: true,
                            disabled: is_verifying,
                            autocomplete: "one-time-code",
                        }
                        Button { r#type: "submit", disabled: is_verifying, "Verify Email" }
                    }

                    match feedback() {
                        MessageKind::Error(msg) => rsx! {
                            div { class: "p-3 bg-red-50 border border-red-200 text-sm rounded text-red-700 text-left",
                                "{msg}"
                            }
                        },
                        MessageKind::None => rsx! {},
                    }

                    p { class: "text-xs text-gray-400 mt-3",
                        "Didn't receive the code? Check your spam folder or try signing up again."
                    }
                }
            }
        }

        SignUpState::Verifying => {
            let display_email = email();
            rsx! {
                div { class: "flex flex-col gap-4 p-4 max-w-sm mx-auto text-center",
                    div { class: "text-4xl mb-2", "✉️" }
                    h2 { class: "text-2xl font-bold", "Check Your Email" }
                    p { class: "text-gray-600 mt-1", "We sent a verification code to:" }
                    p { class: "font-semibold text-black mt-1", "{display_email}" }

                    form {
                        onsubmit: handle_verify,
                        class: "flex flex-col gap-3 mt-4",
                        Input {
                            r#type: "text",
                            placeholder: "Enter 6-digit code",
                            value: "{otp_code}",
                            oninput: move |e: Event<FormData>| otp_code.set(e.value()),
                            class: "text-center text-xl tracking-widest font-mono",
                            maxlength: "6",
                            required: true,
                            disabled: true,
                            autocomplete: "one-time-code",
                        }
                        Button { r#type: "submit", disabled: true, "Verifying..." }
                    }

                    match feedback() {
                        MessageKind::Error(msg) => rsx! {
                            div { class: "p-3 bg-red-50 border border-red-200 text-sm rounded text-red-700 text-left",
                                "{msg}"
                            }
                        },
                        MessageKind::None => rsx! {},
                    }

                    p { class: "text-xs text-gray-400 mt-3",
                        "Didn't receive the code? Check your spam folder or try signing up again."
                    }
                }
            }
        }

        // ── Verified Success Screen ──
        SignUpState::Verified => {
            rsx! {
                div { class: "flex flex-col gap-4 p-4 max-w-sm mx-auto text-center",
                    div { class: "text-4xl mb-2", "✅" }
                    h2 { class: "text-2xl font-bold text-green-700", "Email Verified!" }
                    p { class: "text-gray-600 mt-2",
                        "Your account is ready. You can now sign in with your credentials."
                    }
                    div { class: "bg-green-50 border border-green-200 rounded-lg p-4 mt-4 text-sm text-green-800",
                        "Switch to the "
                        span { class: "font-bold", "Sign In" }
                        " tab above to log in."
                    }
                }
            }
        }

        // ── Sign Up Form (default + loading) ──
        _ => {
            let is_loading = state() == SignUpState::Loading;
            rsx! {
                Card {
                    CardHeader {
                        CardTitle { "Create a new account" }
                        CardDescription { "Enter your details below to create a new account" }
                    }
                    CardContent {
                        form {
                            onsubmit: handle_signup,
                            class: "flex flex-col gap-4",
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "signup_name", "Name" }
                                Input {
                                    id: "signup_name",
                                    r#type: "text",
                                    placeholder: "John Doe",
                                    value: "{name}",
                                    oninput: move |e: Event<FormData>| name.set(e.value()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "signup_email", "Email" }
                                Input {
                                    id: "signup_email",
                                    r#type: "email",
                                    placeholder: "m@example.com",
                                    value: "{email}",
                                    oninput: move |e: Event<FormData>| email.set(e.value()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }
                            div { class: "flex flex-col gap-2",
                                Label { html_for: "signup_password", "Password" }
                                Input {
                                    id: "signup_password",
                                    r#type: "password",
                                    value: "{password}",
                                    oninput: move |e: Event<FormData>| password.set(e.value()),
                                    required: true,
                                    disabled: is_loading,
                                }
                            }
                        }
                        match feedback() {
                            MessageKind::Error(msg) => rsx! {
                                div { class: "p-3 bg-red-50 border border-red-200 text-sm rounded text-red-700", "{msg}" }
                            },
                            MessageKind::None => rsx! {},
                        }
                        CardFooter { class: "pt-4 flex flex-col gap-3",
                            Button {
                                r#type: "submit",
                                class: "w-full",
                                disabled: is_loading,
                                if is_loading {
                                    "Creating account..."
                                } else {
                                    "Sign Up"
                                }
                            }
                            div { class: "text-sm text-center text-gray-500 w-full mt-2",
                                "Already have an account?"
                                Link {
                                    to: crate::routes::PublicRoute::SignIn {
                                    },
                                    class: "ml-1 text-blue-600 hover:underline dark:text-blue-400 font-medium",
                                    "Sign in"
                                }
                            }
                        }
                    }

                }
            }
        }
    }
}
