use crate::components::chat_context::ChatContext;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_fluent::{I18n, move_tr};

#[component]
pub fn SideBody(is_admin: bool) -> impl IntoView {
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");
    let i18n = expect_context::<I18n>();
    let num_questions = 2;
    let faq_questions = (1..=num_questions)
        .map(|i| format!("q-{}", i))
        .collect::<Vec<String>>();
    let (faq_toggled, set_faq_toggled) = signal(true);
    let (obj_toggled, set_obj_toggled) = signal(true);
    //let show_objects = move || obj_toggled.get();
    view! {
        <a on:click=move |_| ctx.clear_history.set(true)>
            <i class="fas fa-edit"></i>
            <span>{move || move_tr!("new-chat")}</span>
        </a>
        <a on:click=move |_| {
            set_faq_toggled.try_update(|value| *value = !*value);
        }>
            <i class="fas fa-book"></i>
            <span>{move || move_tr!("faq")}</span>
        </a>
        <div class="faq-area" class:none=move || faq_toggled.get()>
            {faq_questions
                .into_iter()
                .map(|key| {
                    let question = move || i18n.tr(&key);
                    view! {
                        <a on:click=move |_| ctx.insert_text.set(Some(question()))>
                            <i class="fas fa-question"></i>
                            <span class="faq-question">{question.clone()}</span>
                        </a>
                    }
                })
                .collect_view()}
        </div>
        <a on:click=move |_| { set_obj_toggled.try_update(|value| *value = !*value);}>
            <i class="fas fa-building"></i>
            <span>{move || move_tr!("objects")}</span>
        </a>
        <Show when= move || obj_toggled.get() fallback=|| view! { <Objects/> } >
            view!{}
        </Show>
        <hr />
        <a href="/">
            <i class="fas fa-home"></i>
            <span>{move || move_tr!("home")}</span>
        </a>
        <a href="/play">
            <i class="fas fa-gear"></i>
            <span>{move || move_tr!("play")}</span>
        </a>
        <hr />

        {if is_admin {
            view! {
                <a href="/profile">
                    <i class="fas fa-user"></i>
                    <span>{move || move_tr!("profile")}</span>
                </a>
                <a href="#">
                    <i class="fas fa-users"></i>
                    <span>{move || move_tr!("users")}</span>
                </a>
            }
                .into_any()
        } else {
            ().into_any()
        }}
    }
}
#[component]
fn Objects() -> impl IntoView {
    view! {
        <div class="obj-area">
        <details>
            <summary>
                <span class="node-content">Obj 1</span>
            </summary>
            <div class="leaf">
                <span class="node-content">Building 2.1</span>
            </div>
            <details>
                <summary>
                    <span class="node-content">Building 2.2</span>
                </summary>
                <div class="leaf">
                    <span class="node-content">Room 2.2.1</span>
                </div>
                <div class="leaf">
                    <span class="node-content">Room 2.2.2</span>
                </div>
            </details>
        </details>
        </div>
    }
}
