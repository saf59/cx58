use leptos::prelude::*;
use leptos_fluent::{I18n, Language, leptos_fluent, move_tr, tr};

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
            {move || tr!(
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
            <select
                class="lang-list"
                name="languages"
                prop:value=move || i18n.language.get().id.to_string()
                on:change=move |event| {
                    let selected = event_target_value(&event);
                    if let Some(lang) = i18n.languages.iter().find(|lang| lang.id == selected) {
                        i18n.language.set(lang);
                    }
                }
            >
                {i18n.languages.iter().map(|lang| {
                    let lang_id = lang.id.to_string();
                    view! {
                        <option class="lang-item" value=lang_id.clone()>
                            {lang_id.clone()}
                        </option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </div>
    }
}

fn render_language(lang: &'static Language) -> impl IntoView {
    let i18n = expect_context::<I18n>();

    view! {
        <div>
            <label for=lang.id.to_string()>{lang.name}</label>
            <input
                id=lang.id.to_string()
                value=lang.id.to_string()
                name="language"
                checked=move || i18n.language.get() == lang
                on:click=move |_| i18n.language.set(lang)
                type="radio"
            />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    fn message_ids(source: &str) -> BTreeSet<&str> {
        source
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with('.') {
                    return None;
                }
                let (id, _) = line.split_once('=')?;
                let id = id.trim();
                (!id.is_empty()
                    && id
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')))
                .then_some(id)
            })
            .collect()
    }

    #[test]
    fn english_and_german_message_ids_match() {
        let english = message_ids(include_str!("../../locales/en/main.ftl"));
        let german = message_ids(include_str!("../../locales/de/main.ftl"));

        assert_eq!(english, german);
    }
}
