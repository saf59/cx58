use js_sys::Function;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos::web_sys::Headers;
use leptos::*;
use leptos::logging::log;
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
    pub role: String, // "user" or "llm"
    pub content: String,
}

#[component]
pub fn Chat() -> impl IntoView {
    let (history, set_history) = signal(vec![]);
    let (input, set_input) = signal("".to_string());
    let (is_loading, set_is_loading) = signal(false);
    let chat_history_ref = NodeRef::new();
	let form_ref:NodeRef<html::Form> = NodeRef::new();
    let client = reqwest_wasm::Client::new();
    let session_id = uuid::Uuid::new_v4().to_string();
    // Scroll down Effect
    Effect::new(move |_| {
        // Just tracking the change in history.
        history.with(|_| ());
        let el: Option<HtmlDivElement> = chat_history_ref.get();
        if let Some(chat_history_el) = el {
            let child: Option<Element> = chat_history_el.last_element_child();
            if let Some(last_message_el) = child {
                let scroll_options = ScrollIntoViewOptions::new();
                scroll_options.set_behavior(ScrollBehavior::Smooth);
                // Create a callback that will be executed on the next frame.
                let callback = Closure::once_into_js(move |_timestamp: f64| {
                    last_message_el.scroll_into_view_with_scroll_into_view_options(&scroll_options);
                });
                // Planning to call this callback
                let _ = web_sys::window()
                    .unwrap()
                    .request_animation_frame(callback.unchecked_ref::<Function>());
            }
        }
    });
    // append user request
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
        let session_id = session_id.clone();
        // asynchronous task for working with SSE
        spawn_local(async move {
            let headers = Headers::new().unwrap();
            headers.append("Content-Type", "application/json").unwrap();
            let opts = RequestInit::new();
            opts.set_method("POST");
            opts.set_body(&wasm_bindgen::JsValue::from_str(
                &json!({ "prompt": prompt, "session_id": session_id  }).to_string(),
            ));
            opts.set_headers(&headers);

            let request = web_sys::window()
                .unwrap()
                .fetch_with_str_and_init("/api/chat_stream", &opts);

            // Waiting for a response from the server
            let resp_value = JsFuture::from(request).await;
            let response: Response = resp_value.unwrap().dyn_into().unwrap();
            // Error handling!
            if !response.ok() {
                let error_text = response.text().unwrap();
                let err_msg = JsFuture::from(error_text)
                    .await
                    .unwrap()
                    .as_string()
                    .unwrap();
                set_history.update(|h| {
                    h.push(Message {
                        role: "error".to_string(),
                        content: format!("Request failed: {}", err_msg),
                    });
                });
                set_is_loading.set(false);
                return;
            }

            let body = response.body().unwrap();
            let reader = body.get_reader();
            let reader = reader.dyn_into::<ReadableStreamDefaultReader>().unwrap();

            loop {
                // Read the next chunk
                let chunk = JsFuture::from(reader.read()).await.unwrap();
                let done = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("done"))
                    .unwrap()
                    .as_bool()
                    .unwrap();

                if done {
                    tracing::info!("SSE stream finished by client.");
                    break;
                }

                let value = js_sys::Reflect::get(&chunk, &wasm_bindgen::JsValue::from_str("value"))
                    .unwrap();
                let array = js_sys::Uint8Array::from(value);
                let text = String::from_utf8_lossy(&array.to_vec()).into_owned();

                // SSE parse
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.is_empty() {
                            continue;
                        }
                        // Add token to history
                        set_history.update(|h| {
                            if let Some(last_msg) = h.last_mut()
                                && last_msg.role == "llm"
                            {
                                last_msg.content.push_str(data);
                                return;
                            }
                            h.push(Message {
                                role: "llm".to_string(),
                                content: format!("<p>{:?}</p>", data),
                            });
                        });
                    }
                }
            }
            set_is_loading.set(false);
        });

        set_input.set("".to_string());
    }};
    // Stop action: POST /api/stop
    let stop = move |_| {
        let client = client.clone();
		let session_id = session_id.clone();
        wasm_bindgen_futures::spawn_local(async move {
			log!("STOP CLICKED");
			let payload = serde_json::json!({
			    "session_id": session_id
			}).to_string();
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
                {// (derived signal)
                move || {
                    history
                        .get()
                        .into_iter()
                        .map(|message| {
                            let is_user = message.role == "user";
                            view! {
                                <div
                                    class=if is_user { "message user" } else { "message bot" }
                                    inner_html=message.content
                                />
                            }
                        })
                        .collect_view()
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
                        on:input=move |ev| {
                            set_input.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev:ev::KeyboardEvent| {
                            if ev.key() == "Enter" && !ev.shift_key() {
									log!("Enter CLICKED");
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
                    <button on:click= stop
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
