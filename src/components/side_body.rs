use crate::auth::Auth;
use crate::components::chat_context::ChatContext;
use crate::components::show_tree::DetailsTreeRendererWithContext;
use crate::components::tree::TreeViewerResource;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_fluent::{I18n, move_tr};

#[component]
pub fn SideBody(is_admin: bool) -> impl IntoView {
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");
    let i18n = expect_context::<I18n>();
    let num_questions = 7;
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
                    let question_clone = question.clone();
                    view! {
                        <div class="faq" >
                            <i class="fas fa-question" on:click=move |_| ctx.insert_and_enter.set(Some(question_clone()))></i>
                            <span class="faq-item" on:click=move |_| ctx.insert_text.set(Some(question()))>{question.clone()}</span>
                        </div>
                    }
                })
                .collect_view()}
        </div>

        <a on:click=move |_| {
            set_obj_toggled.try_update(|value| *value = !*value);
        }>
            <i class="fas fa-building"></i>
            <span>{move || move_tr!("objects")}</span>
        </a>

        <Show when=move || obj_toggled.get() fallback=|| view! { <Objects /> }>
            {().into_view()}
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
    let ctx = use_context::<ChatContext>().expect("ChatContext must be provided");
    let auth_signal = use_context::<RwSignal<Auth>>().expect("Auth must be provided");

    let email = auth_signal
        .get_untracked()
        .email()
        .unwrap_or("mock".to_string());

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="error-boundary">
                    <h3>"Error occurred:"</h3>
                    <pre>{move || format!("{:#?}", errors.get())}</pre>
                </div>
            }
        }>
            <TreeViewerResource
                user_id=email
                with_leafs=false
                renderer=move |tree| {
                    tracing::info!("Rendering tree with {} nodes", tree.len());
                    view! {
                        <DetailsTreeRendererWithContext
                            tree=tree
                            on_node_click=move |node_info| {
                                tracing::info!("Node clicked: {:?}", node_info.name);
                                ctx.set_parent(node_info.clone())
                            }
                        />
                    }
                }
            />
        </ErrorBoundary>
    }
}