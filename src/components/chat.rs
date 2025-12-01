use js_sys::Function;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos::web_sys::Headers;
use leptos::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Element, HtmlDivElement, ReadableStreamDefaultReader, RequestInit, Response, ScrollBehavior,
    ScrollIntoViewOptions,
};

#[derive(Clone, Debug)]
pub struct Message {
    pub role: String,    // "user" или "llm"
    pub content: String, // HTML
}

#[component]
pub fn Chat() -> impl IntoView {
    let (history, set_history) = signal(vec![]);
    let (input, set_input) = signal("".to_string());
    let (is_loading, set_is_loading) = signal(false);
    let chat_history_ref = NodeRef::new();
    let form_ref: NodeRef<html::Form> = NodeRef::new();
    let client = reqwest_wasm::Client::new();
    let session_id = uuid::Uuid::new_v4().to_string();

    // Скролл при изменении истории
    Effect::new(move |_| {
        history.with(|_| ());
        let el: Option<HtmlDivElement> = chat_history_ref.get();
        if let Some(chat_history_el) = el {
            if let Some(last_message_el) = chat_history_el.last_element_child() {
                let scroll_options = ScrollIntoViewOptions::new();
                scroll_options.set_behavior(ScrollBehavior::Smooth);
                let callback = Closure::once_into_js(move |_timestamp: f64| {
                    last_message_el.scroll_into_view_with_scroll_into_view_options(&scroll_options);
                });
                let _ = web_sys::window()
                    .unwrap()
                    .request_animation_frame(callback.unchecked_ref::<Function>());
            }
        }
    });

    // Submit
    let on_submit = {
        let session_id = session_id.clone();
        move |ev: ev::SubmitEvent| {
            ev.prevent_default();
            let prompt = input.get();
            if prompt.is_empty() || is_loading.get() {
                return;
            }
            set_is_loading.set(true);

            set_history.update(|h| {
                h.push(Message {
                    role: "user".to_string(),
                    content: prompt.clone(),
                });
            });

            let session_id_clone = session_id.clone();

            spawn_local(async move {
                let headers = Headers::new().unwrap();
                headers.append("Content-Type", "application/json").unwrap();
                let opts = RequestInit::new();
                opts.set_method("POST");
                opts.set_body(&wasm_bindgen::JsValue::from_str(
                    &json!({ "prompt": prompt, "session_id": session_id_clone }).to_string(),
                ));
                opts.set_headers(&headers);

                let request = web_sys::window()
                    .unwrap()
                    .fetch_with_str_and_init("/api/chat_stream", &opts);
                let resp_value = JsFuture::from(request).await.unwrap();
                let response: Response = resp_value.dyn_into().unwrap();

                if !response.ok() {
                    let error_text = JsFuture::from(response.text().unwrap())
                        .await
                        .unwrap()
                        .as_string()
                        .unwrap();
                    set_history.update(|h| {
                        h.push(Message {
                            role: "error".to_string(),
                            content: format!("Request failed: {}", error_text),
                        });
                    });
                    set_is_loading.set(false);
                    return;
                }

                let body = response.body().unwrap();
                let reader = body.get_reader();
                let reader = reader.dyn_into::<ReadableStreamDefaultReader>().unwrap();
                let mut current_event: Option<String> = None;

                loop {
                    let chunk = JsFuture::from(reader.read()).await.unwrap();
                    let done =
                        js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("done"))
                            .unwrap()
                            .as_bool()
                            .unwrap();
                    if done {
                        break;
                    }

                    let value =
                        js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("value"))
                            .unwrap();
                    let array = js_sys::Uint8Array::from(value);
                    let text = String::from_utf8_lossy(&array.to_vec()).into_owned();

                    for raw_line in text.lines() {
                        if raw_line.is_empty() {
                            continue;
                        }

                        if let Some(evt) = raw_line.strip_prefix("event: ") {
                            current_event = Some(evt.to_string());
                            continue;
                        }

                        if let Some(data) = raw_line.strip_prefix("data: ") {
                            match current_event.as_deref() {
                                // Стриминг текста посимвольно
                                None | Some("chunk") | Some("replay") => {
                                    // data - это просто текст
                                    let text_chunk = data.to_string();
                                    if !text_chunk.is_empty() {
                                        set_history.update(|h| {
                                            if let Some(last) = h.last_mut() {
                                                if last.role == "llm" {
                                                    last.content.push_str(&text_chunk);
                                                } else {
                                                    h.push(Message {
                                                        role: "llm".to_string(),
                                                        content: text_chunk,
                                                    });
                                                }
                                            } else {
                                                h.push(Message {
                                                    role: "llm".to_string(),
                                                    content: text_chunk,
                                                });
                                            }
                                        });
                                    }
                                }

                                // Финальный JSON с objects
                                Some("json") => {
                                    let parsed: serde_json::Value = match serde_json::from_str(data)
                                    {
                                        Ok(v) => v,
                                        Err(_) => continue,
                                    };

                                    let mut html = String::new();

                                    // Объекты - показываем в конце
                                    if let Some(objects) =
                                        parsed.get("objects").and_then(|v| v.as_array())
                                    {
                                        html.push_str("<ul>");
                                        for arr in objects {
                                            if let Some(items) = arr.as_array() {
                                                for item in items {
                                                    // Обрабатываем массивы вида ["type","word"],["index",36.78]
                                                    if let Some(arr_item) = item.as_array() {
                                                        if arr_item.len() >= 2 {
                                                            let key =
                                                                arr_item[0].as_str().unwrap_or("");
                                                            let value = &arr_item[1];
                                                            html.push_str(&format!(
                                                                "<li><strong>{}:</strong> {}</li>",
                                                                key, value
                                                            ));
                                                        }
                                                    } else if let Some(url) =
                                                        item.get("url").and_then(|v| v.as_str())
                                                    {
                                                        // Если есть поле url - показываем картинку
                                                        html.push_str(&format!(
                                                            "<li><img src=\"{}\"/></li>",
                                                            url
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        html.push_str("</ul>");
                                    }

                                    if !html.is_empty() {
                                        set_history.update(|h| {
                                            // Добавляем objects к последнему сообщению
                                            if let Some(last) = h.last_mut() {
                                                if last.role == "llm" {
                                                    last.content.push_str(&html);
                                                }
                                            }
                                        });
                                    }
                                }

                                Some("on_complete") => {
                                    set_is_loading.set(false);
                                }

                                Some("on_stop") => {
                                    let reason = data;
                                    set_history.update(|h| {
                                        h.push(Message {
                                            role: "system".to_string(),
                                            content: format!("<i>⏹ Chat stopped: {}</i>", reason),
                                        });
                                    });
                                    set_is_loading.set(false);
                                }

                                Some("error") => {
                                    let err = data;
                                    set_history.update(|h| {
                                        h.push(Message {
                                            role: "error".to_string(),
                                            content: format!("❌ {}", err),
                                        });
                                    });
                                    set_is_loading.set(false);
                                }

                                _ => {}
                            }

                            current_event = None;
                        }
                    }
                }

                set_is_loading.set(false);
            });

            set_input.set("".to_string());
        }
    };

    // STOP request
    let stop = move |_| {
        let client = client.clone();
        let session_id = session_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let payload = serde_json::json!({ "session_id": session_id }).to_string();
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            let url = format!("{}/api/stop", origin);
            let _ = client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;
        });
    };

    view! {
        <div class="chat-container">
            <div class="chat-history" node_ref=chat_history_ref>
                { move || {
                    history.get().into_iter().map(|message| {
                        let is_user = message.role == "user";
                        view! {
                            <div
                                class=if is_user { "message user" } else { "message bot" }
                                inner_html=message.content
                            />
                        }
                    }).collect_view()
                }}
            </div>

            <div class="chat-input">
                <i
                    class=(["loader"], move || is_loading.get())
                    class=(["none"], move || !is_loading.get())
                />
                <form class="chat-input-form" on:submit=on_submit node_ref=form_ref>
                    <textarea
                        name="chat-input-name"
                        prop:value=input
                        on:input=move |ev| set_input.set(event_target_value(&ev))
                        on:keydown=move |ev:ev::KeyboardEvent| {
                            if ev.key() == "Enter" && !ev.shift_key() {
                                ev.prevent_default();
                                if let Some(form) = form_ref.get() {
                                    let _ = form.request_submit();
                                }
                            }
                        }
                        placeholder="Ask me anything..."
                        class="input-zone"
                        disabled=is_loading
                    />
                    <button on:click=stop
                        class="input-submit"
                        class=(["fa","fa-stop-circle"], move || is_loading.get())
                        class=(["none"], move || !is_loading.get())
                    />
                    <button
                        type="submit"
                        class="input-submit"
                        class=(["fa", "fa-arrow-up"], move || !is_loading.get())
                        class=(["none"], move || is_loading.get())
                    />
                </form>
            </div>
        </div>
    }
}
