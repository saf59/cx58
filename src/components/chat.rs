use crate::components::chat_context::ChatContext;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos::*;
use leptos_fluent::{move_tr, I18n};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::JsFuture;

#[cfg(feature = "hydrate")]
use web_sys::{
    HtmlDivElement, ReadableStreamDefaultReader, RequestInit, Response, ScrollBehavior,
    ScrollIntoViewOptions,
};
use crate::auth::Auth;
use crate::components::node_info_display::NodeInfoDisplay;
use crate::components::show_carusel::CarouselRenderer;
use crate::components::show_tree::DetailsTreeRendererWithContext;
use crate::components::tree::{Tree, TreeNode, build_tree, NodeInfo};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Llm,
    System,
    Error,
}

impl MessageRole {
    fn css_class(&self) -> &'static str {
        match self {
            Self::User => "message user",
            Self::Llm => "message bot",
            Self::System => "message system",
            Self::Error => "message error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    ObjectTree(Vec<Tree>),
    DocumentTree(Vec<Tree>),
    Description(DescriptionData),
    Comparison(ComparisonData),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DescriptionData {
    pub description: String,
    pub fields: Vec<(String, String)>, // (key, value) pairs
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparisonData {
    pub fields: Vec<(String, String)>, // (key, value) pairs
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: MessageContent,
}

impl Message {
    fn new(role: MessageRole, content: MessageContent) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            role,
            content,
        }
    }

    fn new_text(role: MessageRole, text: String) -> Self {
        Self::new(role, MessageContent::Text(text))
    }
}

#[component]
pub fn Chat() -> impl IntoView {
    let (history, set_history) = signal(Vec::<Message>::new());
    let (input, set_input) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (chat_state, set_chat_state) = signal(String::new());
    let chat_history_ref = NodeRef::new();
    let form_ref = NodeRef::<html::Form>::new();
    let chat_id = uuid::Uuid::now_v7().to_string();
    let i18n = expect_context::<I18n>();
    let auth_signal = use_context::<RwSignal<Auth>>().expect("Auth must be provided");
    let user_id = auth_signal.get_untracked().email().unwrap_or("mock".to_string());
    let ctx = use_context::<ChatContext>().expect("ChatContext not provided");
    let delete_node_info = Callback::new(move |node_info: NodeInfo| {
        ctx.delete_node_info(node_info)
    });

    // Subscribe to context
    Effect::new(move |_| {
            if ctx.clear_history.get() {
                set_history.set(Vec::new());
                set_chat_state.set(String::new());
                ctx.clear_history.set(false);
            }
    });

    Effect::new(move |_| {
            if let Some(text) = ctx.insert_text.get() {
                set_input.set(text);
                ctx.insert_text.set(None);
            }
    });
    // Browser close handler
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")] {
            if let Some(window) = web_sys::window() {
                let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::BeforeUnloadEvent| {
                    if let Some(navigator) = web_sys::window().map(|w| w.navigator()) {
                        let _ = send_beacon_stop(&navigator);
                    }
                }) as Box<dyn FnMut(_)>);

                let _ = window.add_event_listener_with_callback(
                    "beforeunload",
                    closure.as_ref().unchecked_ref()
                );

                closure.forget();
            }
        }
    });

    // Autoscroll when history changes
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")] {
            history.track();
            let history_ref: Option<HtmlDivElement> = chat_history_ref.get();
            if let Some(el) = history_ref {
                let scroll_options = ScrollIntoViewOptions::new();
                scroll_options.set_behavior(ScrollBehavior::Smooth);
                let history_el: HtmlDivElement = el.clone();
                let _ = gloo_timers::callback::Timeout::new(50, move || {
                    if let Some(last) = history_el.last_element_child() {
                        last.scroll_into_view_with_scroll_into_view_options(&scroll_options);
                    }
                })
                    .forget();
            }
        }
    });
    let owner = Owner::current();
    // Submit handler
    let on_submit = {
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
            if let Some(owner_ref) = owner.as_ref() {
                let chat_id = chat_id.clone();
                let language = i18n.language.get().id.to_string();
                let user_id = user_id.clone();
                let owner_clone = owner_ref.clone();

                spawn_local(async move {
                    #[cfg(feature = "hydrate")] {
                        let _ = owner_clone.with(move || async move {
                            if let Err(e) = handle_stream(
                                prompt,
                                chat_id,
                                set_history,
                                set_is_loading,
                                set_chat_state,
                                language,
                                user_id,
                                ctx
                            )
                                .await
                            {
                                set_history.update(|h| {
                                    h.push(Message::new_text(
                                        MessageRole::Error,
                                        format!("⚠ Connection error: {}", e),
                                    ));
                                });
                                set_is_loading.set(false);
                            }
                        }).await;
                    }
                });
            }
        }
    };

    // Stop handler
    let on_stop = move |_| {
        #[cfg(feature = "hydrate")]
        {
            spawn_local(async move {
                if let Err(e) = send_stop_request().await {
                    tracing::error!("Failed to stop: {}", e);
                }
            });
        }
    };

    view! {
        <div class="chat-container">
            <div class="chat-history" node_ref=chat_history_ref>
                {move || {
                    history
                        .get()
                        .into_iter()
                        .map(|message| {
                            view! { <MessageRenderer message=message /> }
                        })
                        .collect_view()
                }}
            </div>

            <div class="chat-state" class:hidden=move || chat_state.get().is_empty()>
                <i class="fa fa-spinner fa-spin"></i>
                <span inner_html=move || chat_state.get()></span>
            </div>

            <div class="chat-input">
                <i class="loader" class:none=move || !is_loading.get() />
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
                        class=(["fa", "fa-arrow-up"], move || !is_loading.get() && !input.get().is_empty() )
                        class:none=move || is_loading.get() || input.get().is_empty()
                        data-descr=move_tr!("start")
                    />
                </form>
            </div>
        </div>
    }
}

#[component]
fn MessageRenderer(message: Message) -> impl IntoView {
    let css_class = message.role.css_class();
    let ctx = use_context::<ChatContext>().expect("Context lost");

    match message.content {
        MessageContent::Text(text) => {
            view! { <div class=css_class inner_html=text /> }.into_any()
        }
        MessageContent::ObjectTree(tree) => {
            view! {
                <div class=css_class>
                    <DetailsTreeRendererWithContext
                        tree=tree
                        on_node_click=move |node_info| {
                            tracing::info!("Node clicked: {:?}", node_info.name);
                            ctx.insert_text
                                .set(Some(node_info.name.unwrap_or("(unnamed)".to_string())));
                        }
                    />
                </div>
            }.into_any()
        }
        MessageContent::DocumentTree(tree) => {
            view! {
                <div class=css_class>
                    <CarouselRenderer tree=tree />
                </div>
            }.into_any()
        }
        MessageContent::Description(data) => {
            view! {
                <div class=css_class>
                    <div class="description-content">
                        <h4>"Description"</h4>
                        <p>{data.description}</p>
                        {data
                            .fields
                            .into_iter()
                            .map(|(key, value)| {
                                view! {
                                    <div class="description-field">
                                        <h5>{key}</h5>
                                        <p>{value}</p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            }.into_any()
        }
        MessageContent::Comparison(data) => {
            view! {
                <div class=css_class>
                    <div class="comparison-content">
                        <h4>"Comparison"</h4>
                        {data
                            .fields
                            .into_iter()
                            .map(|(key, value)| {
                                view! {
                                    <div class="comparison-field">
                                        <h5>{key}</h5>
                                        <p>{value}</p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            }.into_any()
        }
    }
}

// Helpers
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "hydrate")]
async fn handle_stream(
    prompt: String,
    chat_id: String,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
    set_chat_state: WriteSignal<String>,
    language: String,
    email: String,
    context: ChatContext
) -> Result<(), String> {

    let window = web_sys::window().ok_or("No window")?;

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers error: {:?}", e))?;
    headers
        .append("Content-Type", "application/json")
        .map_err(|e| format!("Header append error: {:?}", e))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);

    let parent = context.parent.get_untracked();
    let prev_leaf = context.prev_leaf.get_untracked();
    let next_leaf = context.next_leaf.get_untracked();

    let mut body_map = serde_json::Map::new();
    body_map.insert("message".to_string(), json!(prompt));
    body_map.insert("user_id".to_string(), json!(email));
    body_map.insert("chat_id".to_string(), json!(chat_id));
    body_map.insert("language".to_string(), json!(language));

    if let Some(p) = parent {
        body_map.insert("object_id".to_string(), json!(p));
    }

    if let Some(pl) = prev_leaf {
        body_map.insert("prev_leaf".to_string(), json!(pl));
    }

    if let Some(nl) = next_leaf {
        body_map.insert("next_leaf".to_string(), json!(nl));
    }

    let body = serde_json::Value::Object(body_map).to_string();

    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let request = window.fetch_with_str_and_init("/api/chat_stream", &opts);

    let resp_value = JsFuture::from(request)
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let response: Response = resp_value
        .dyn_into()
        .map_err(|_| "Invalid response".to_string())?;

    if !response.ok() {
        let status = response.status();
        let error_text = if let Ok(text_promise) = response.text() {
            JsFuture::from(text_promise)
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| format!("HTTP {}", status))
        } else {
            format!("HTTP {}", status)
        };

        set_history.update(|h| {
            h.push(Message::new_text(
                MessageRole::Error,
                format!("Request failed: {}", error_text),
            ));
        });
        set_is_loading.set(false);
        return Err(error_text);
    }

    let body = response.body().ok_or("No body")?;
    let reader = body.get_reader();
    let reader: ReadableStreamDefaultReader = reader
        .dyn_into()
        .map_err(|_| "Invalid reader".to_string())?;

    process_stream(reader, set_history, set_is_loading, set_chat_state).await
}
#[cfg(feature = "hydrate")]
async fn process_stream(
    reader: ReadableStreamDefaultReader,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
    set_chat_state: WriteSignal<String>,
) -> Result<(), String> {
    let mut current_event: Option<String> = None;
    let mut buffer = String::new();

    loop {
        let chunk = JsFuture::from(reader.read())
            .await
            .map_err(|e| format!("Read error: {:?}", e))?;

        let done = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("done"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("value"))
            .map_err(|_| "No value in chunk")?;

        let array = js_sys::Uint8Array::from(value);
        let bytes = array.to_vec();
        let text = String::from_utf8_lossy(&bytes);

        buffer.push_str(&text);

        // Process lines
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer.drain(..=newline_pos);

            if line.is_empty() {
                continue;
            }

            if let Some(evt) = line.strip_prefix("event: ") {
                current_event = Some(evt.to_string());
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                process_sse_event(&current_event, data, set_history, set_is_loading, set_chat_state);
                current_event = None;
            }
        }
    }

    set_is_loading.set(false);
    set_chat_state.set(String::new());
    Ok(())
}

fn process_sse_event(
    event: &Option<String>,
    data: &str,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
    set_chat_state: WriteSignal<String>,
) {
    match event.as_deref() {
        None | Some("chunk") | Some("replay") => {
            if !data.is_empty() {
                set_history.update(|h| {
                    append_or_create_text_message(h, data.to_string());
                });
            }
        }

        Some("coordinator_thinking") => {
            set_chat_state.set(format!("🤔 {}", data));
        }

        Some("object_chunk") => {
            if let Ok(nodes) = serde_json::from_str::<Vec<TreeNode>>(data) {
                let tree = build_tree(nodes);
                set_history.update(|h| {
                    h.push(Message::new(MessageRole::Llm, MessageContent::ObjectTree(tree)));
                });
            }
        }

        Some("document_chunk") => {
            if let Ok(nodes) = serde_json::from_str::<Vec<TreeNode>>(data) {
                let tree = build_tree(nodes);
                set_history.update(|h| {
                    h.push(Message::new(MessageRole::Llm, MessageContent::DocumentTree(tree)));
                });
            }
        }

        Some("description_chunk") => {
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(data) {
                let mut fields = Vec::new();
                let description = json_data.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if let Some(obj) = json_data.as_object() {
                    for (key, value) in obj.iter() {
                        if key != "description"
                            && let Some(val_str) = value.as_str() {
                                fields.push((capitalize(key), val_str.to_string()));
                            }
                    }
                }

                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::Description(DescriptionData { description, fields }),
                    ));
                });
            }
        }

        Some("comparison_chunk") => {
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(data) {
                let mut fields = Vec::new();

                if let Some(obj) = json_data.as_object() {
                    for (key, value) in obj.iter() {
                        if let Some(val_str) = value.as_str() {
                            fields.push((capitalize(key), val_str.to_string()));
                        }
                    }
                }

                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::Comparison(ComparisonData { fields }),
                    ));
                });
            }
        }

        Some("completed") | Some("on_complete") => {
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }

        Some("on_stop") | Some("cancelled") => {
            set_history.update(|h| {
                h.push(Message::new_text(
                    MessageRole::System,
                    format!("<i>ℹ Chat stopped: {}</i>", data),
                ));
            });
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }

        Some("error") => {
            set_history.update(|h| {
                h.push(Message::new_text(MessageRole::Error, format!("⚠ {}", data)));
            });
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }

        _ => {}
    }
}

fn append_or_create_text_message(history: &mut Vec<Message>, content: String) {
    if let Some(last) = history.last_mut()
        && last.role == MessageRole::Llm
            && let MessageContent::Text(ref mut text) = last.content {
                text.push_str(&content);
                return;
    }
    history.push(Message::new_text(MessageRole::Llm, content));
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

// Stop request - session_id extracted from cookie on server
#[cfg(feature = "hydrate")]
async fn send_stop_request() -> Result<(), String> {
    tracing::info!("send_stop_request called!");
    let window = web_sys::window().ok_or("No window")?;
    let origin = window.location().origin().map_err(|_| "No origin")?;

    let url = format!("{}/api/stop", origin);
    let payload = json!({}).to_string();

    let headers = web_sys::Headers::new().map_err(|_| "Headers error")?;
    headers.append("Content-Type", "application/json")
        .map_err(|_| "Header append error")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&payload));
    opts.set_credentials(web_sys::RequestCredentials::SameOrigin);

    let request = web_sys::Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Request error: {:?}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Stop error: {:?}", e))?;

    let resp: web_sys::Response = resp_value.dyn_into()
        .map_err(|_| "Response cast error")?;

    if !resp.ok() {
        return Err(format!("Stop failed: {}", resp.status()));
    }

    Ok(())
}

// SendBeacon for beforeunload - cookies sent automatically
#[cfg(feature = "hydrate")]
fn send_beacon_stop(navigator: &web_sys::Navigator) -> Result<bool, wasm_bindgen::JsValue> {
    let url = "/api/stop";
    tracing::info!("Stop beacon called!");
    let success = navigator.send_beacon_with_opt_str(url, None)?;
    Ok(success)
}

