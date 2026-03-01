use leptos_macro::{component, view};
use serde::{Deserialize, Serialize};
use leptos::IntoView;
use leptos::prelude::{CollectView, IntoAny};
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextRequest {
    pub prompt: String,
    pub suggestions: Vec<String>,
}

#[component]
pub fn ContextRequestRenderer(data: ContextRequest) -> impl IntoView {
    view! {
        <div class="context-request border-left-red">
            <p class="context-request__prompt">{data.prompt.clone()}</p>
            {if data.suggestions.len() == 1 {
                view! { <p class="context-request__hint">{data.suggestions[0].clone()}</p> }
                    .into_any()
            } else if !data.suggestions.is_empty() {
                view! {
                    <ul class="context-request__suggestions">
                        {data
                            .suggestions
                            .iter()
                            .map(|s| view! { <li>{s.clone()}</li> })
                            .collect_view()}
                    </ul>
                }
                    .into_any()
            } else {
                view! { <></> }.into_any()
            }}
        </div>
    }
}