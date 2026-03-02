#![cfg(not(feature = "ssr"))]
use crate::components::args;
use crate::components::chat_context::ChatContext;
use crate::components::chat_data::{ComparisonData, ContextRequest, DescriptionData};
use crate::components::chat_types::{Message, MessageContent, MessageRole};
use crate::components::tree::NodeWithLeaf;
use leptos::leptos_dom::log;
use leptos::logging;
use leptos::prelude::{GetUntracked, Set, Update, WriteSignal};
use leptos_fluent::I18n;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStreamDefaultReader, RequestInit, Response};

// Helpers
#[allow(clippy::too_many_arguments)]
pub async fn handle_stream(
    prompt: String,
    chat_id: String,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
    set_chat_state: WriteSignal<String>,
    language: String,
    email: String,
    context: ChatContext,
    i18n: I18n,
) -> Result<(), String> {
    use serde_json::json;

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
        body_map.insert("object_id".to_string(), json!(p.id));
    }

    if let Some(pl) = prev_leaf {
        body_map.insert("prev_leaf".to_string(), json!(pl.id));
    }

    if let Some(nl) = next_leaf {
        body_map.insert("next_leaf".to_string(), json!(nl.id));
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
                i18n.tr_with_args(
                    "chat-error-request-failed",
                    &args!["error" => error_text.clone()],
                ),
            ));
        });
        set_is_loading.set(false);
        return Err(error_text);
    }

    let body = response.body().ok_or("No body")?;
    let reader = body.get_reader();
    let reader: ReadableStreamDefaultReader = reader
        .dyn_into()
        .map_err(|_| i18n.tr("chat-error-invalid-reader"))?;

    process_stream(reader, set_history, set_is_loading, set_chat_state, i18n).await
}
async fn process_stream(
    reader: ReadableStreamDefaultReader,
    set_history: WriteSignal<Vec<Message>>,
    set_is_loading: WriteSignal<bool>,
    set_chat_state: WriteSignal<String>,
    i18n: I18n,
) -> Result<(), String> {
    let mut current_event: Option<String> = None;
    let mut buffer = String::new();

    loop {
        let chunk = JsFuture::from(reader.read()).await.map_err(|e| {
            i18n.tr_with_args(
                "chat-error-read-error",
                &args!["error" => format!("{:?}", e)],
            )
        })?;

        let done = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("done"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("value"))
            .map_err(|_| i18n.tr("chat-error-no-chunk-value"))?;

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
                process_sse_event(
                    &current_event,
                    data,
                    set_history,
                    set_is_loading,
                    set_chat_state,
                    i18n,
                );
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
    i18n: I18n,
) {
    use crate::components::tree::{build_tree, TreeNode};

    match event.as_deref() {
        None | Some("chunk") | Some("replay") => {
            if !data.is_empty() {
                set_history.update(|h| {
                    append_or_create_text_message(h, data.to_string());
                });
            }
        }

        Some("progress") => {
            set_chat_state.set(data.to_string());
        }

        Some("object") => {
            if let Ok(nodes) = serde_json::from_str::<Vec<TreeNode>>(data) {
                let tree = build_tree(nodes);
                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::ObjectTree(tree),
                    ));
                });
            }
        }

        Some("report_list") => match serde_json::from_str::<Vec<NodeWithLeaf>>(data) {
            Ok(nodes) => {
                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::DocumentTree(nodes),
                    ));
                });
            }
            Err(e) => {
                logging::error!("Failed to parse document chunk: {}", e);
                log!("{}", data);
            }
        },

        Some("description") => match serde_json::from_str::<Vec<DescriptionData>>(data) {
            Ok(json_data) => {
                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::Description(Box::new(json_data)),
                    ));
                });
            }
            Err(e) => {
                logging::error!("Failed to parse description chunk: {}", e);
                log!("{}", data);
            }
        },

        Some("comparison") => match serde_json::from_str::<ComparisonData>(data) {
            Ok(json_data) => {
                set_history.update(|h| {
                    h.push(Message::new(
                        MessageRole::Llm,
                        MessageContent::Comparison(json_data),
                    ));
                });
            }
            Err(e) => {
                logging::error!("Failed to parse comparision: {}", e);
                log!("{}", data);
            }
        },
        Some("context_request") => {
            match serde_json::from_str::<ContextRequest>(data) {
                Ok(json_data) => {
                    //log!("Received comparision:\n{:?}", &json_data);
                    set_history.update(|h| {
                        h.push(Message::new(
                            MessageRole::Llm,
                            MessageContent::ContextRequest(json_data),
                        ));
                    });
                }
                Err(e) => {
                    logging::error!("Failed to parse context request: {}", e);
                    log!("{}", data);
                }
            }
        }
        Some("completed") | Some("on_complete") => {
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }
        Some("on_stop") | Some("cancelled") => {
            let reason = match data {
                "by_user" => i18n.tr("chat-stop-by-user"),
                "timeout" => i18n.tr("chat-stop-timeout"),
                "max_tokens" => i18n.tr("chat-stop-max-tokens"),
                "transport_error" => i18n.tr("chat-stop-transport-error"),
                other => other.to_string(),
            };
            set_history.update(|h| {
                h.push(Message::new_text(MessageRole::System, reason));
            });
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }
        Some("error") => {
            let msg = if let Some(rest) = data.strip_prefix("llm-error|") {
                let mut parts = rest.splitn(2, '|');
                let status = parts.next().unwrap_or("");
                let detail = parts.next().unwrap_or("");
                i18n.tr_with_args(
                    "chat-llm-error",
                    &args![
                        "status" => status,
                        "detail" => detail,
                    ],
                )
            } else if let Some(rest) = data.strip_prefix("transport-error|") {
                i18n.tr_with_args(
                    "chat-transport-error",
                    &args![
                        "error" => rest,
                    ],
                )
            } else {
                // Fallback: plain text from agent (legacy or unknown format)
                i18n.tr_with_args("chat-error-server", &args!["error" => data])
            };

            set_history.update(|h| {
                h.push(Message::new_text(MessageRole::Error, msg));
            });
            set_is_loading.set(false);
            set_chat_state.set(String::new());
        }

        _ => {}
    }
}

// Stop request - session_id extracted from cookie on server
pub fn send_stop_beacon() -> Result<bool, String> {
    tracing::info!("Stop beacon called!");

    web_sys::window()
        .ok_or("No window")?
        .navigator()
        .send_beacon_with_opt_str("/api/stop", None)
        .map_err(|e| format!("Beacon error: {:?}", e))
}

fn append_or_create_text_message(history: &mut Vec<Message>, content: String) {
    if let Some(last) = history.last_mut()
        && last.role == MessageRole::Llm
        && let MessageContent::Text(ref mut text) = last.content
    {
        text.push_str(&content);
        return;
    }
    history.push(Message::new_text(MessageRole::Llm, content));
}
