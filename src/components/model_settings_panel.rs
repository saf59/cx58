use crate::model_settings::{
    ModelChange, ModelSettings, ModelsResponse, OllamaModelInfo, UpdateModelsRequest,
    UpdateModelsResponse,
};
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use leptos_fluent::move_tr;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::wasm_bindgen::{JsCast, JsValue};
use web_sys::{Request, RequestInit, RequestMode, Response};

#[component]
pub fn ModelSettingsPanel(user_id: String) -> impl IntoView {
    let (reload, set_reload) = signal(0_u32);
    let (initialized, set_initialized) = signal(false);
    let (vision_model, set_vision_model) = signal(String::new());
    let (text_model, set_text_model) = signal(String::new());
    let (chat_model, set_chat_model) = signal(String::new());
    let (same, set_same) = signal(false);
    let (saving, set_saving) = signal(false);
    let (status, set_status) = signal(String::new());
    let (changes, set_changes) = signal(Vec::<ModelChange>::new());

    let models_resource = LocalResource::new({
        let user_id = user_id.clone();
        move || {
            let user_id = user_id.clone();
            let _ = reload.get();
            async move { fetch_models(&user_id).await }
        }
    });

    Effect::new(move |_| {
        if initialized.get_untracked() {
            return;
        }

        if let Some(Ok(data)) = models_resource.get() {
            apply_settings(
                &data.current,
                set_vision_model,
                set_text_model,
                set_chat_model,
            );
            set_initialized.set(true);
        }
    });

    view! {
        <div class="models-panel">
            <Suspense fallback=move || view! { <div class="models-status">{move_tr!("models-loading")}</div> }>
                {move || {
                    models_resource
                        .get()
                        .map(|result| match result {
                            Ok(data) => {
                                let vision_models = models_for_role(&data.models, "vision");
                                let text_models = models_for_role(&data.models, "text");
                                let chat_models = models_for_role(&data.models, "tools");

                                view! {
                                    <div class="models-grid">
                                        <ModelSelect
                                            id="models-vision-model"
                                            name="vision_model"
                                            label=move_tr!("models-vision").get()
                                            value=vision_model
                                            set_value=set_vision_model
                                            models=vision_models
                                            current=data.current.vision_model.clone()
                                        >
                                            <label
                                                class="models-link-toggle"
                                                class:active=move || same.get()
                                                title=move || move_tr!("models-link-title").get()
                                                for="models-same-model"
                                            >
                                                <input
                                                    id="models-same-model"
                                                    name="same_model"
                                                    aria-label=move || move_tr!("models-link-title").get()
                                                    type="checkbox"
                                                    checked=move || same.get()
                                                    on:change=move |event| set_same.set(checkbox_checked(&event))
                                                />
                                                <i class="fas fa-link"></i>
                                            </label>
                                        </ModelSelect>
                                        <ModelSelect
                                            id="models-text-model"
                                            name="text_model"
                                            label=move_tr!("models-text").get()
                                            value=text_model
                                            set_value=set_text_model
                                            models=text_models
                                            current=data.current.text_model.clone()
                                            disabled=Signal::derive(move || same.get())
                                        />
                                        <ModelSelect
                                            id="models-chat-model"
                                            name="chat_model"
                                            label=move_tr!("models-chat").get()
                                            value=chat_model
                                            set_value=set_chat_model
                                            models=chat_models
                                            current=data.current.chat_model.clone()
                                            disabled=Signal::derive(move || same.get())
                                        />
                                    </div>
                                    <div class="models-actions">
                                        <button
                                            type="button"
                                            class="models-save"
                                            disabled=move || saving.get()
                                            on:click={
                                                let user_id = user_id.clone();
                                                move |_| {
                                                    let saved_label = move_tr!("models-saved").get_untracked();
                                                    start_save(
                                                        user_id.clone(),
                                                        saved_label,
                                                        vision_model,
                                                        text_model,
                                                        chat_model,
                                                        same,
                                                        saving,
                                                        set_vision_model,
                                                        set_text_model,
                                                        set_chat_model,
                                                        set_saving,
                                                        set_status,
                                                        set_changes,
                                                    );
                                                }
                                            }
                                        >
                                            <i class="fas fa-save"></i>
                                            <span>{move || if saving.get() { move_tr!("models-saving").get() } else { move_tr!("models-save").get() }}</span>
                                        </button>
                                        <button
                                            type="button"
                                            class="models-refresh"
                                            on:click=move |_| {
                                                set_initialized.set(false);
                                                set_reload.update(|value| *value += 1);
                                            }
                                            title=move || move_tr!("models-refresh").get()
                                        >
                                            <i class="fas fa-rotate"></i>
                                        </button>
                                    </div>
                                    <ModelStatus status=status changes=changes />
                                }
                                    .into_any()
                            }
                            Err(e) => view! {
                                <div class="models-status models-error">{e}</div>
                            }
                                .into_any(),
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ModelSelect(
    id: &'static str,
    name: &'static str,
    label: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    models: Vec<OllamaModelInfo>,
    current: String,
    #[prop(default = Signal::derive(|| false))] disabled: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let options = options_with_current(models, current);
    let children = children.map(|children| children());

    view! {
        <div class="models-row">
            <label for=id>{label}</label>
            <div class="models-control">
                <select
                    id=id
                    name=name
                    prop:value=move || value.get()
                    disabled=move || disabled.get()
                    on:change=move |event| set_value.set(event_target_value(&event))
                >
                    {options
                        .into_iter()
                        .map(|model| {
                            let name = model.name.clone();
                            view! {
                                <option value=name.clone()>{name.clone()}</option>
                            }
                        })
                        .collect_view()}
                </select>
                {children}
            </div>
        </div>
    }
}

#[component]
fn ModelStatus(status: ReadSignal<String>, changes: ReadSignal<Vec<ModelChange>>) -> impl IntoView {
    view! {
        <div class="models-status" class:none=move || status.get().is_empty() && changes.get().is_empty()>
            <Show when=move || !status.get().is_empty()>
                <p>{move || status.get()}</p>
            </Show>
            <Show when=move || !changes.get().is_empty()>
                <ul>
                    {move || {
                        changes
                            .get()
                            .into_iter()
                            .map(|change| {
                                let class = if change.applied { "applied" } else { "rejected" };
                                view! {
                                    <li class=class>
                                        <span>{change.role}</span>
                                        <small>{change.reason}</small>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Show>
        </div>
    }
}

fn apply_settings(
    settings: &ModelSettings,
    set_vision_model: WriteSignal<String>,
    set_text_model: WriteSignal<String>,
    set_chat_model: WriteSignal<String>,
) {
    set_vision_model.set(settings.vision_model.clone());
    set_text_model.set(settings.text_model.clone());
    set_chat_model.set(settings.chat_model.clone());
}

fn start_save(
    user_id: String,
    saved_label: String,
    vision_model: ReadSignal<String>,
    text_model: ReadSignal<String>,
    chat_model: ReadSignal<String>,
    same: ReadSignal<bool>,
    saving: ReadSignal<bool>,
    set_vision_model: WriteSignal<String>,
    set_text_model: WriteSignal<String>,
    set_chat_model: WriteSignal<String>,
    set_saving: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    set_changes: WriteSignal<Vec<ModelChange>>,
) {
    if saving.get_untracked() {
        return;
    }

    set_saving.set(true);
    set_status.set(String::new());
    set_changes.set(Vec::new());

    let request = if same.get_untracked() {
        UpdateModelsRequest {
            vision_model: non_empty(vision_model.get_untracked()),
            text_model: None,
            chat_model: None,
            same: Some(true),
        }
    } else {
        UpdateModelsRequest {
            vision_model: non_empty(vision_model.get_untracked()),
            text_model: non_empty(text_model.get_untracked()),
            chat_model: non_empty(chat_model.get_untracked()),
            same: Some(false),
        }
    };

    spawn_local(async move {
        match update_models(&user_id, &request).await {
            Ok(response) => {
                apply_settings(
                    &response.current,
                    set_vision_model,
                    set_text_model,
                    set_chat_model,
                );
                set_changes.set(response.changes);
                set_status.set(saved_label);
            }
            Err(e) => set_status.set(e),
        }
        set_saving.set(false);
    });
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn models_for_role(models: &[OllamaModelInfo], role: &str) -> Vec<OllamaModelInfo> {
    models
        .iter()
        .filter(|model| supports_role(model, role))
        .cloned()
        .collect()
}

fn supports_role(model: &OllamaModelInfo, role: &str) -> bool {
    let required = match role {
        "vision" => "vision",
        "text" => "completion",
        "tools" => "tools",
        _ => role,
    };

    model
        .capabilities
        .iter()
        .any(|capability| capability == required)
}

fn options_with_current(mut models: Vec<OllamaModelInfo>, current: String) -> Vec<OllamaModelInfo> {
    if !current.is_empty() && !models.iter().any(|model| model.name == current) {
        models.insert(
            0,
            OllamaModelInfo {
                name: current,
                size: None,
                modified_at: None,
                capabilities: Vec::new(),
                family: None,
                parameter_size: None,
                quantization_level: None,
            },
        );
    }
    models
}

async fn fetch_models(user_id: &str) -> Result<ModelsResponse, String> {
    let url = format!("/api/models/{}", user_id);
    fetch_json(&url).await
}

async fn update_models(
    user_id: &str,
    request: &UpdateModelsRequest,
) -> Result<UpdateModelsResponse, String> {
    let body = serde_json::to_string(request)
        .map_err(|e| format!("Failed to serialize model settings: {e}"))?;
    let url = format!("/api/models/{}", user_id);
    send_json(&url, "PUT", Some(body)).await
}

async fn fetch_json<T>(url: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    send_json(url, "GET", None).await
}

async fn send_json<T>(url: &str, method: &str, body: Option<String>) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let window = web_sys::window().ok_or_else(|| "No window available".to_string())?;
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);
    if let Some(body) = body {
        opts.set_body(&JsValue::from_str(&body));
    }

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| format!("Failed to set accept header: {:?}", e))?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set content type: {:?}", e))?;

    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to convert fetch response".to_string())?;

    let status = response.status();
    let text_value = JsFuture::from(
        response
            .text()
            .map_err(|e| format!("Failed to read response body: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to read response body: {:?}", e))?;
    let text = text_value.as_string().unwrap_or_default();

    if !(200..300).contains(&status) {
        return Err(extract_error(&text).unwrap_or_else(|| {
            if text.is_empty() {
                format!("Request failed: {status}")
            } else {
                format!("Request failed: {status}: {text}")
            }
        }));
    }

    serde_json::from_str(&text).map_err(|e| format!("Failed to deserialize response: {e}"))
}

fn extract_error(text: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(text).ok()?;
    let error = value.get("error")?;

    if let Some(message) = error.as_str() {
        return Some(message.to_string());
    }

    if let Some(message) = error.get("message").and_then(|value| value.as_str()) {
        if let Some(code) = error.get("code").and_then(|value| value.as_str()) {
            return Some(format!("{code}: {message}"));
        }
        return Some(message.to_string());
    }

    Some(error.to_string())
}

fn checkbox_checked(event: &web_sys::Event) -> bool {
    event
        .target()
        .and_then(|target| js_sys::Reflect::get(&target, &JsValue::from_str("checked")).ok())
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}
