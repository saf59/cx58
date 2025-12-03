use crate::components::chat_context::ChatContext;
use leptos::attr::href;
use leptos::html::A;
use leptos::prelude::*;
use leptos::{IntoView, component, view};

#[component]
pub fn SideBody(is_admin: bool) -> impl IntoView {
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");
    let faq_questions = vec!["Tell me story", "Say 10"];
    let (faq_toggled, set_faq_toggled) = signal(true);

    view! {
        <a on:click=move |_| ctx.clear_history.set(true)>
            <i class="fas fa-edit"></i>
            <span>New chat</span>
        </a>
        <a on:click=move |_| {
            set_faq_toggled.try_update(|value| *value = !*value);
        }>
            <i class="fas fa-book"></i>
            <span>FAQ</span>
        </a>
        <div class="faq-buttons" class:none=move || faq_toggled.get()>
            {faq_questions
                .into_iter()
                .map(|question| {
                    let question = question.to_string();
                    view! {
                        <a on:click=move |_| {
                            ctx.insert_text.set(Some(question.clone()));
                        }>
                            <i class="fas fa-question"></i>
                            <span>{question.clone()}</span>
                        </a>
                    }
                })
                .collect_view()}
        </div>
        <a href="#">
            <i class="fas fa-building"></i>
            <span>Objects</span>
        </a>

        <hr />
        {if is_admin {
            view! {
                <a href="/">
                    <i class="fas fa-home"></i>
                    <span>Home</span>
                </a>
                <a href="/profile">
                    <i class="fas fa-user"></i>
                    <span>Profile</span>
                </a>
                <a href="/play">
                    <i class="fas fa-language"></i>
                    <span>Play</span>
                </a>
                <a href="#">
                    <i class="fas fa-users"></i>
                    <span>Users</span>
                </a>
            }
                .into_any()
        } else {
            ().into_any()
        }}
    }
}
