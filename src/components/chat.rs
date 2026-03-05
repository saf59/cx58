use crate::components::{
    chat_context::ChatContext,
    chat_types::{Message, MessageRole},
    node_info_display::NodeInfoDisplay,
    tree::NodeInfo,
    message_renderer::MessageRenderer,
};
use leptos::prelude::*;
use leptos::*;
use leptos_fluent::{move_tr};

#[cfg(not(feature = "ssr"))]
use {
    crate::auth::Auth,
    crate::components::{
        args,
        chat_client::{handle_stream, send_stop_beacon},
    },
    leptos::reactive::spawn_local,
    leptos_fluent::I18n,
    wasm_bindgen::JsCast,
    web_sys::HtmlDivElement,
};

#[component]
pub fn Chat() -> impl IntoView {
    #[allow(unused)]
    let (history, set_history) = signal(Vec::<Message>::new());
    let (input, set_input) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (chat_state, set_chat_state) = signal(String::new());
    let chat_history_ref = NodeRef::new();
    let form_ref = NodeRef::<html::Form>::new();
    #[cfg(not(feature = "ssr"))]
    let (chat_id, i18n, user_id) = {
        let chat_id = uuid::Uuid::now_v7().to_string();
        let i18n = expect_context::<I18n>();
        let auth_signal = use_context::<RwSignal<Auth>>().expect("Auth must be provided");
        let user_id = auth_signal
            .get_untracked()
            .email()
            .unwrap_or("mock".to_string());
        (chat_id, i18n, user_id)
    };
    let ctx = use_context::<ChatContext>().expect("ChatContext not provided");
    let delete_node_info =
        Callback::new(move |node_info: NodeInfo| ctx.delete_node_info(node_info));

    // Clear history handler
    Effect::new(move |_| {
        if ctx.clear_history.get() {
            set_history.set(Vec::new());
            set_chat_state.set(String::new());
            ctx.clear_history.set(false);
        }
    });
    // Insert text and submit handler
    Effect::new(move |_| {
        if let Some(text) = ctx.insert_and_enter.get() {
            set_input.set(text);
            ctx.insert_and_enter.set(None);
            if let Some(form) = form_ref.get() {
                let _ = form.request_submit();
            }
        }
    });
    // Insert text without submitting handler
    Effect::new(move |_| {
        if let Some(text) = ctx.insert_text.get() {
            set_input.set(text);
            ctx.insert_text.set(None);
        }
    });
    // Browser close handler
    #[cfg(not(feature = "ssr"))]
    {
        if let Some(window) = web_sys::window() {
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                move |_event: web_sys::BeforeUnloadEvent| {
                    let _ = send_stop_beacon();
                },
            ) as Box<dyn FnMut(_)>);
            let _ = window.add_event_listener_with_callback(
                "beforeunload",
                closure.as_ref().unchecked_ref(),
            );
            closure.forget();
        }
    }
    // Autoscroll when history changes handler
    Effect::new(move |_| {
        history.track();
        #[cfg(not(feature = "ssr"))]
        {
            let history_ref: Option<HtmlDivElement> = chat_history_ref.get();
            if let Some(el) = history_ref {
                let _ = gloo_timers::callback::Timeout::new(100, move || {
                    el.set_scroll_top(el.scroll_height());
                })
                    .forget();
            }
        }
    });

    #[cfg(not(feature = "ssr"))]
    let owner = Owner::current();
    // Submit handler
    let on_submit = {
        #[cfg(not(feature = "ssr"))]
        let chat_id = chat_id.clone();
        move |ev: ev::SubmitEvent| {
            ev.prevent_default();
            let prompt = input.get();
            if prompt.trim().is_empty() || is_loading.get() {
                return;
            }
            set_is_loading.set(true);
            set_history.update(|h| {
                h.push(Message::new_text(MessageRole::User, prompt.clone()));
            });
            set_input.set(String::new());
            #[cfg(not(feature = "ssr"))]
            {
                if let Some(owner_ref) = owner.as_ref() {
                    let chat_id = chat_id.clone();
                    let language = i18n.language.get().id.to_string();
                    let user_id = user_id.clone();
                    let owner_clone = owner_ref.clone();

                    spawn_local(async move {
                        let _ = owner_clone
                            .with(move || async move {
                                if let Err(e) = handle_stream(
                                    prompt,
                                    chat_id,
                                    set_history,
                                    set_is_loading,
                                    set_chat_state,
                                    language,
                                    user_id,
                                    ctx,
                                    i18n,
                                )
                                .await
                                {
                                    set_history.update(|h| {
                                        h.push(Message::new_text(
                                            MessageRole::Error,
                                            i18n.tr_with_args(
                                                "chat-error-connection",
                                                &args!["error" => e],
                                            ),
                                        ));
                                    });
                                    set_is_loading.set(false);
                                }
                            })
                            .await;
                    });
                }
            }
        }
    };
    // Stop handler
    let on_stop = move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            spawn_local(async move {
                if let Err(e) = send_stop_beacon() {
                    tracing::error!("Failed to stop: {}", e);
                }
            });
        }
    };

    // chat history and input UI
    view! {
        <div class="chat-container">
            <div class="chat-history" node_ref=chat_history_ref>
                {
                    move || {
                    history
                        .get()
                        .into_iter()
                        .map(|message| {
                            //view! { <Fake /> }
                            view! { <MessageRenderer message=message /> }
                        })
                        .collect_view()
                }}
            </div>
            <div class="chat-input">
                <div class="chat-state" class:hidden=move || chat_state.get().is_empty()>
                    <i class="fa fa-spinner fa-spin"></i>
                    <span inner_html=move || chat_state.get()></span>
                </div>
                <form class="chat-input-form" on:submit=on_submit node_ref=form_ref>
                    <textarea
                        name="chat-input-name"
                        prop:value=move || input.get()
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=move |ev: ev::KeyboardEvent| {
                            if ev.key() == "Enter" && !ev.shift_key() {
                                ev.prevent_default();
                                if let Some(form) = form_ref.get() {
                                    let _ = form.request_submit();
                                }
                            }
                        }
                        placeholder=move_tr!("ask-me-anything")
                        class="input-zone"
                        prop:disabled=is_loading
                    />
                    <div class="node-info-section">
                        <NodeInfoDisplay node_signal=ctx.parent on_node_click=delete_node_info />
                        <div class="node-info-leafs">
                            <NodeInfoDisplay
                                node_signal=ctx.prev_leaf
                                on_node_click=delete_node_info
                            />
                            <NodeInfoDisplay
                                node_signal=ctx.next_leaf
                                on_node_click=delete_node_info
                            />
                        </div>
                    </div>

                    <button
                        type="button"
                        on:click=on_stop
                        class="input-submit"
                        class=(["fa", "fa-stop-circle"], move || is_loading.get())
                        class=(["none"], move || !is_loading.get())
                        data-descr=move_tr!("stop")
                    />
                    <button
                        type="submit"
                        class="input-submit"
                        class=(
                            ["fa", "fa-arrow-up"],
                            move || !is_loading.get() && !input.get().is_empty(),
                        )
                        class:none=move || is_loading.get() || input.get().is_empty()
                        data-descr=move_tr!("start")
                    />
                </form>
            </div>
        </div>
        <div class="sb-footer">{move_tr!("chat-footer")}</div>
    }
}

#[component]
fn Fake() -> impl IntoView {
    view! {
        <div class="centered bg_oidc">
            <h1>"Fake Page"</h1>
            <p>"This is a placeholder page."</p>
        </div>
    }
}
