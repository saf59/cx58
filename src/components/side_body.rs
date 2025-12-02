use crate::components::chat_context::ChatContext;
use leptos::prelude::{ClassAttribute, CollectView, ElementChild, OnAttribute, Set, use_context};
use leptos::{IntoView, component, view};

#[component]
pub fn SideBody() -> impl IntoView {
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");
    let faq_questions = vec!["Tell me story", "Say 10"];

    view! {
        <a on:click=move |_| ctx.clear_history.set(true)>
            <i class="fas fa-edit"></i>
            <span>New chat</span>
        </a>
        <a href="#">
            <i class="fas fa-book"></i>
            <span>FAQ</span>
        </a>
        <a href="#">
            <i class="fas fa-building"></i>
            <span>Objects</span>
        </a>
        <a href="#">
            <i class="fas fa-users"></i>
            <span>Users</span>
        </a>
            <div class="faq-buttons">
                        {faq_questions.into_iter().map(|question| {
                            let question = question.to_string();
                            view! {
                                <a on:click=move |_| {
                                        ctx.insert_text.set(Some(question.clone()));
                                    }>
                                <i class="fas fa-question"></i>
                                <span>{question.clone()}</span>
                                </a>
                            }
                        }).collect_view()}
            </div>
    }
}
