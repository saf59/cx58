use crate::components::chat_data::ContextRequest;
use leptos::prelude::ElementChild;
use leptos::prelude::IntoAny;
use leptos::prelude::*;
use leptos::IntoView;
use leptos_macro::{component, view};

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
