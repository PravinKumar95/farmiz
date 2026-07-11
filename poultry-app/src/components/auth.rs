use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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

#[component]
pub fn SignIn() -> Element {
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut message = use_signal(|| "".to_string());

    let handle_submit = move |evt: Event<FormData>| {
        evt.stop_propagation();
        spawn(async move {
            let payload = SignInPayload {
                email: email(),
                password: password(),
            };

            let backend_url = option_env!("BACKEND_URL").unwrap_or("http://127.0.0.1:9000/lambda-url/poultry-backend");
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/api/auth/signin", backend_url))
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                                message.set(format!("Signed in! Token: {}...", &token[..10]));
                            } else {
                                message.set("Signed in, but no token returned.".into());
                            }
                        }
                    } else {
                        message.set(format!("Error: {}", resp.status()));
                    }
                }
                Err(e) => message.set(format!("Request failed: {}", e)),
            }
        });
    };

    rsx! {
        div {
            class: "flex flex-col gap-4 p-4 max-w-sm mx-auto",
            h2 { class: "text-2xl font-bold", "Sign In" }
            form {
                onsubmit: handle_submit,
                class: "flex flex-col gap-3",
                input {
                    r#type: "email",
                    placeholder: "Email",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                    class: "border p-2 rounded text-black",
                    required: true
                }
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    class: "border p-2 rounded text-black",
                    required: true
                }
                button {
                    r#type: "submit",
                    class: "bg-blue-500 text-white p-2 rounded hover:bg-blue-600 transition-colors",
                    "Sign In"
                }
            }
            if !message().is_empty() {
                div { class: "p-2 bg-gray-100 text-sm mt-2 rounded text-black", "{message}" }
            }
        }
    }
}

#[component]
pub fn SignUp() -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut message = use_signal(|| "".to_string());

    let handle_submit = move |evt: Event<FormData>| {
        evt.stop_propagation();
        spawn(async move {
            let payload = SignUpPayload {
                name: name(),
                email: email(),
                password: password(),
            };

            let backend_url = option_env!("BACKEND_URL").unwrap_or("http://127.0.0.1:9000/lambda-url/poultry-backend");
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{}/api/auth/signup", backend_url))
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        message.set("Account created! You can now sign in.".into());
                    } else {
                        let err = resp.text().await.unwrap_or_default();
                        message.set(format!("Error: {}", err));
                    }
                }
                Err(e) => message.set(format!("Request failed: {}", e)),
            }
        });
    };

    rsx! {
        div {
            class: "flex flex-col gap-4 p-4 max-w-sm mx-auto",
            h2 { class: "text-2xl font-bold", "Sign Up" }
            form {
                onsubmit: handle_submit,
                class: "flex flex-col gap-3",
                input {
                    r#type: "text",
                    placeholder: "Name",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    class: "border p-2 rounded text-black",
                    required: true
                }
                input {
                    r#type: "email",
                    placeholder: "Email",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                    class: "border p-2 rounded text-black",
                    required: true
                }
                input {
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    class: "border p-2 rounded text-black",
                    required: true
                }
                button {
                    r#type: "submit",
                    class: "bg-green-500 text-white p-2 rounded hover:bg-green-600 transition-colors",
                    "Sign Up"
                }
            }
            if !message().is_empty() {
                div { class: "p-2 bg-gray-100 text-sm mt-2 rounded text-black", "{message}" }
            }
        }
    }
}
