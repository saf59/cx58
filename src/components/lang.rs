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
        //check_translations: true,
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
#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let i18n = expect_context::<I18n>();
    view! {
        <div class="lang">
            <select class="lang-list" name="languages"
            on:change=move |event| {
                let selected = event_target_value(&event);
                //let selected: String = event.target().value();
                let find =i18n.languages.iter().find(|lang| lang.id == selected);
                if let Some(lang) = find {
                    i18n.language.set(lang);
                }
            }
            >
            {move || {
                let selected = i18n.language.get();
                i18n.languages.iter().map(|lang| render_id(lang, selected)).collect::<Vec<_>>()
            }}
            </select>
        </div>
    }
}

fn render_id(lang: &'static Language, selected: &'static Language) -> impl IntoView {
    view! {
            <option selected=move || lang == selected
                class="lang-item"
                value={lang}
                >{move || lang.id.to_string()}</option>
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
                checked=move ||i18n.language.get() == lang
                on:click=move |_| i18n.language.set(lang)
                type="radio"
            />
        </div>
    }
}
