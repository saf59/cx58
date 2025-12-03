// src/lib.rs
use leptos::prelude::*;
use leptos_fluent::{I18n, Language, leptos_fluent, move_tr};

#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    leptos_fluent! {
        children: children(),
        locales: "./locales",
        default_language: "en",
        set_language_to_cookie: true,
        initial_language_from_cookie: true,
        initial_language_from_navigator: true,
        initial_language_from_navigator_to_cookie: true,
        initial_language_from_accept_language_header: true,
        cookie_name: "lf-lang",
    }
}

#[component]
pub fn LanguageSelector() -> impl IntoView {
    let i18n = expect_context::<I18n>();

    view! {
        <p>{move_tr!("select-a-language")}</p>
        <fieldset>
            {move || {
                i18n.languages.iter().map(|lang| render_language(lang)).collect::<Vec<_>>()
            }}
        </fieldset>
        <p>
            {move_tr!(
                "language-selected-is",
                 { "lang" => i18n.language.get().name }
            )}
        </p>
    }
}

fn render_language(lang: &'static Language) -> impl IntoView {
    let i18n = expect_context::<I18n>();

    // Passed as atrribute, `Language` is converted to their code,
    // so `<input id=lang` becomes `<input id=lang.id.to_string()`
    view! {
        <div>
            <label for=lang>{lang.name}</label>
            <input
                id=lang
                value=lang
                name="language"
                checked=i18n.language.get() == lang
                on:click=move |_| i18n.language.set(lang)
                type="radio"
            />
        </div>
    }
}
