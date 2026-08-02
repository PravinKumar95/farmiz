use dioxus::prelude::*;

#[component]
pub fn ThemeToggle() -> Element {
    let mut is_dark = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let eval_res = document::eval(
                "return localStorage.getItem('theme') === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches);"
            );
            if let Ok(val) = eval_res.await {
                if let Some(dark_bool) = val.as_bool() {
                    is_dark.set(dark_bool);
                    if dark_bool {
                        let _ = document::eval(
                            "document.documentElement.setAttribute('data-theme', 'dark'); document.documentElement.classList.add('dark'); document.body.classList.add('dark');"
                        );
                    } else {
                        let _ = document::eval(
                            "document.documentElement.setAttribute('data-theme', 'light'); document.documentElement.classList.remove('dark'); document.body.classList.remove('dark');"
                        );
                    }
                }
            }
        });
    });

    let toggle_theme = move |_| {
        let next_dark = !is_dark();
        is_dark.set(next_dark);
        spawn(async move {
            if next_dark {
                let _ = document::eval(
                    "document.documentElement.setAttribute('data-theme', 'dark'); document.documentElement.classList.add('dark'); document.body.classList.add('dark'); localStorage.setItem('theme', 'dark');"
                );
            } else {
                let _ = document::eval(
                    "document.documentElement.setAttribute('data-theme', 'light'); document.documentElement.classList.remove('dark'); document.body.classList.remove('dark'); localStorage.setItem('theme', 'light');"
                );
            }
        });
    };

    rsx! {
        button {
            r#type: "button",
            onclick: toggle_theme,
            class: "flex items-center gap-1.5 px-3 py-1.5 rounded-lg border text-xs font-medium bg-stone-100 hover:bg-stone-200 dark:bg-stone-800 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 border-stone-300 dark:border-stone-700 transition-colors cursor-pointer",
            aria_label: "Toggle theme",
            title: if is_dark() { "Switch to Light Mode" } else { "Switch to Dark Mode" },
            if is_dark() {
                span { class: "text-sm", "🌙" }
                span { class: "font-semibold", "Dark" }
            } else {
                span { class: "text-sm", "☀️" }
                span { class: "font-semibold", "Light" }
            }
        }
    }
}
