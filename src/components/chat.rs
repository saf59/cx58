use crate::components::chat_context::ChatContext;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlDivElement, ReadableStreamDefaultReader, RequestInit, Response, ScrollBehavior,
    ScrollIntoViewOptions,
};

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
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Llm => "llm",
            Self::System => "system",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content,
        }
    }
}

#[component]
pub fn Chat() -> impl IntoView {
    let (history, set_history) = signal(Vec::<Message>::new());
    let (input, set_input) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let chat_history_ref = NodeRef::new();
    let form_ref = NodeRef::<html::Form>::new();
    let session_id = uuid::Uuid::new_v4().to_string();

    //  Subscribe to context
    if let Some(ctx) = use_context::<ChatContext>() {
        // Следим за сигналом очистки
        Effect::new(move |_| {
            if ctx.clear_history.get() {
                set_history.set(Vec::new());
                ctx.clear_history.set(false);
            }
        });

        // Следим за сигналом вставки текста
        Effect::new(move |_| {
            if let Some(text) = ctx.insert_text.get() {
                set_input.set(text);
                ctx.insert_text.set(None);
            }
        });
    }

    // Autoscroll when history changes
    Effect::new(move |_| {
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
    });

    // Submit handler
    let on_submit = {
        let session_id = session_id.clone();
        move |ev: ev::SubmitEvent| {
            ev.prevent_default();

            let prompt = input.get();
            if prompt.trim().is_empty() || is_loading.get() {
                return;
            }

            set_is_loading.set(true);
            set_history.update(|h| {
                h.push(Message::new(MessageRole::User, prompt.clone()));
            });
            set_input.set(String::new());

            let session_id = session_id.clone();
            spawn_local(async move {
                if let Err(e) = handle_stream(prompt, session_id, set_history, set_is_loading).await
                {
                    set_history.update(|h| {
                        h.push(Message::new(
                            MessageRole::Error,
                            format!("❌ Connection error: {}", e),
                        ));
                    });
                    set_is_loading.set(false);
                }
            });
        }
    };

    // Stop handler
    let on_stop = {
        let session_id = session_id.clone();
        move |_| {
            let session_id = session_id.clone();
            spawn_local(async move {
                let _ = send_stop_request(&session_id).await;
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
                            view! {
                                <div class=message.role.css_class() inner_html=message.content />
                            }
                        })
                        .collect_view()
                }}
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
                        placeholder="Ask me anything..."
                        class="input-zone"
                        prop:disabled=is_loading
                    />
                    <button
                        type="button"
                        on:click=on_stop
                        class="input-submit"
                        class=(["fa", "fa-stop-circle"], move || is_loading.get())
                        class=(["none"], move || !is_loading.get())
                    />
                    <button
                        type="submit"
                        class="input-submit"
                        class=(["fa", "fa-arrow-up"], move || !is_loading.get())
                        class:none=move || is_loading.get()
                    />
                </form>
            </div>
        </div>
    }
}

// Helpers
async fn handle_stream(
    prompt: String,
    session_id: String,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;

    let headers = web_sys::Headers::new().map_err(|e| format!("Headers error: {:?}", e))?;
    headers
        .append("Content-Type", "application/json")
        .map_err(|e| format!("Header append error: {:?}", e))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);

    let body = json!({
        "prompt": prompt,
        "session_id": session_id
    })
    .to_string();

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
            h.push(Message::new(
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

    process_stream(reader, set_history, set_is_loading).await
}

async fn process_stream(
    reader: ReadableStreamDefaultReader,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
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
                process_sse_event(&current_event, data, set_history, set_is_loading);
                current_event = None;
            }
        }
    }

    set_is_loading.set(false);
    Ok(())
}

fn process_sse_event(
    event: &Option<String>,
    data: &str,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
) {
    match event.as_deref() {
        None | Some("chunk") | Some("replay") => {
            if !data.is_empty() {
                set_history.update(|h| {
                    append_or_create_llm_message(h, data.to_string());
                });
            }
        }

        Some("json") => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data)
                && let Some(html) = extract_objects_html(&parsed)
            {
                set_history.update(|h| {
                    if let Some(last) = h.last_mut()
                        && last.role == MessageRole::Llm
                    {
                        last.content.push_str(&html);
                    }
                });
            }
        }

        Some("on_complete") => {
            set_is_loading.set(false);
        }

        Some("on_stop") => {
            set_history.update(|h| {
                h.push(Message::new(
                    MessageRole::System,
                    format!("<i>⏹ Chat stopped: {}</i>", data),
                ));
            });
            set_is_loading.set(false);
        }

        Some("error") => {
            set_history.update(|h| {
                h.push(Message::new(MessageRole::Error, format!("❌ {}", data)));
            });
            set_is_loading.set(false);
        }

        _ => {}
    }
}

fn append_or_create_llm_message(history: &mut Vec<Message>, content: String) {
    if let Some(last) = history.last_mut()
        && last.role == MessageRole::Llm
    {
        last.content.push_str(&content);
        return;
    }
    history.push(Message::new(MessageRole::Llm, content));
}

fn extract_objects_html(parsed: &serde_json::Value) -> Option<String> {
    let objects = parsed.get("objects")?.as_array()?;
    let mut html = String::from("<ul>");

    for arr in objects {
        if let Some(items) = arr.as_array() {
            for item in items {
                if let Some(arr_item) = item.as_array() {
                    if arr_item.len() >= 2 {
                        let key = arr_item[0].as_str().unwrap_or("");
                        let value = &arr_item[1];
                        html.push_str(&format!("<li><strong>{}:</strong> {}</li>", key, value));
                    }
                } else if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                    html.push_str(&format!("<li><img src=\"{}\"/></li>", url));
                }
            }
        }
    }

    html.push_str("</ul>");
    Some(html)
}

async fn send_stop_request(session_id: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let origin = window
        .location()
        .origin()
        .map_err(|_| "No origin".to_string())?;

    let url = format!("{}/api/stop", origin);
    let payload = json!({ "session_id": session_id }).to_string();

    let headers = web_sys::Headers::new().map_err(|_| "Headers error")?;
    headers
        .append("Content-Type", "application/json")
        .map_err(|_| "Header append error")?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&payload));

    let request = window.fetch_with_str_and_init(&url, &opts);
    JsFuture::from(request)
        .await
        .map_err(|e| format!("Stop request error: {:?}", e))?;

    Ok(())
}
