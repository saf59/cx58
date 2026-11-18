use serde::{Deserialize, Serialize};
use uuid::Uuid;

const CX_ID_BATCH_SIZE: usize = 1000;

pub(crate) fn name_with_cx_id(name: &str, cx_id: Option<&str>) -> String {
    match cx_id {
        Some(cx_id) => format!("{name} / {cx_id}"),
        None => name.to_string(),
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CxIdBatch {
    pub node_ids: Vec<Uuid>,
}

#[derive(Deserialize, Serialize)]
pub struct CxIdValue {
    // An omitted field cannot clear an existing ID.
    #[serde(deserialize_with = "Deserialize::deserialize")]
    pub cx_id: Option<String>,
}

pub(crate) fn use_cx_ids(
    node_ids: impl Fn() -> Vec<Uuid> + 'static,
) -> leptos::prelude::Callback<Uuid, Option<String>> {
    use crate::components::chat_context::ChatContext;
    use leptos::prelude::*;
    use std::collections::HashMap;

    let ctx = expect_context::<ChatContext>();
    let load = Action::new_unsync(|ids: &Vec<Uuid>| {
        let ids = ids.clone();
        async move {
            let mut values = HashMap::new();
            for chunk in ids.chunks(CX_ID_BATCH_SIZE) {
                let batch = CxIdBatch {
                    node_ids: chunk.to_vec(),
                };
                if let Ok(found) =
                    request::<HashMap<Uuid, String>>("/api/cx-ids", "POST", Some(&batch)).await
                {
                    values.extend(found);
                }
            }
            values
        }
    });
    Effect::new(
        move |previous: Option<leptos::reactive::actions::ActionAbortHandle>| {
            if let Some(previous) = previous {
                previous.abort();
            }
            load.dispatch(node_ids())
        },
    );

    Callback::new(move |id| {
        ctx.edited_cx_id(id)
            .unwrap_or_else(|| {
                load.value()
                    .with(|values| values.as_ref().and_then(|values| values.get(&id).cloned()))
            })
            .filter(|value| !value.is_empty())
    })
}

pub(crate) async fn request<T: serde::de::DeserializeOwned>(
    url: &str,
    method: &str,
    body: Option<&impl Serialize>,
) -> Result<T, u16> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response, window};

    const NETWORK_ERROR: u16 = 0;
    let window = window().ok_or(NETWORK_ERROR)?;
    let options = RequestInit::new();
    options.set_method(method);
    if let Some(value) = body {
        let body = serde_json::to_string(value).map_err(|_| NETWORK_ERROR)?;
        options.set_body(&JsValue::from_str(&body));
    }
    let request = Request::new_with_str_and_init(url, &options).map_err(|_| NETWORK_ERROR)?;
    if body.is_some() {
        request
            .headers()
            .set("Content-Type", "application/json")
            .map_err(|_| NETWORK_ERROR)?;
    }
    let response: Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| NETWORK_ERROR)?
        .dyn_into()
        .map_err(|_| NETWORK_ERROR)?;
    if !response.ok() {
        return Err(response.status());
    }
    let json = JsFuture::from(response.json().map_err(|_| NETWORK_ERROR)?)
        .await
        .map_err(|_| NETWORK_ERROR)?;
    serde_wasm_bindgen::from_value(json).map_err(|_| NETWORK_ERROR)
}

#[cfg(feature = "ssr")]
pub mod proxy {
    use super::CxIdValue;
    use crate::{auth_ssr::authenticated_operator, hmac::build_hmac, state::AppState};
    use axum::{
        Json,
        extract::{Path, State},
        http::{StatusCode, header},
        response::{IntoResponse, Response},
    };
    use axum_extra::extract::CookieJar;
    use uuid::Uuid;

    pub async fn read(
        State(state): State<AppState>,
        Path(node_id): Path<Uuid>,
        cookies: CookieJar,
    ) -> Response {
        forward(&state, cookies, node_id, None).await
    }

    pub async fn write(
        State(state): State<AppState>,
        Path(node_id): Path<Uuid>,
        cookies: CookieJar,
        Json(value): Json<CxIdValue>,
    ) -> Response {
        forward(&state, cookies, node_id, Some(value)).await
    }

    pub async fn read_batch(
        State(state): State<AppState>,
        cookies: CookieJar,
        Json(batch): Json<super::CxIdBatch>,
    ) -> Response {
        let result = async {
            if batch.node_ids.len() > super::CX_ID_BATCH_SIZE {
                return Err(StatusCode::BAD_REQUEST);
            }
            let user_id = authenticated_operator(&state, &cookies).await?;
            let body = serde_json::json!({ "user_id": user_id, "node_ids": batch.node_ids });
            request_agent::<std::collections::HashMap<Uuid, String>>(&state, "cx-ids", body).await
        }
        .await;
        ([(header::CACHE_CONTROL, "no-store")], result).into_response()
    }

    async fn forward(
        state: &AppState,
        cookies: CookieJar,
        node_id: Uuid,
        update: Option<CxIdValue>,
    ) -> Response {
        let result = async {
            let user_id = authenticated_operator(state, &cookies).await?;
            let body = match update {
                Some(value) => serde_json::json!({
                    "action": "write", "user_id": user_id, "node_id": node_id, "cx_id": value.cx_id,
                }),
                None => serde_json::json!({
                    "action": "read", "user_id": user_id, "node_id": node_id,
                }),
            };
            request_agent::<CxIdValue>(state, "cx-id", body).await
        }
        .await;
        ([(header::CACHE_CONTROL, "no-store")], result).into_response()
    }

    async fn request_agent<T: serde::de::DeserializeOwned>(
        state: &AppState,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<Json<T>, StatusCode> {
        let body = body.to_string();

        let config = &state.http_client.config.chat_config;
        let (timestamp, signature) = build_hmac(
            config.agent_api_key.as_deref().unwrap_or_default(),
            body.as_bytes(),
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let response = state
            .async_http_client
            .post(format!(
                "{}/agent/{endpoint}",
                config.agent_api_url.trim_end_matches('/')
            ))
            .header("X-Timestamp", timestamp.to_string())
            .header("X-Signature", signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            let error = response.json::<serde_json::Value>().await.ok();
            if error
                .as_ref()
                .and_then(|value| value.pointer("/error/code"))
                == Some(&serde_json::json!("NotFound"))
            {
                return Err(StatusCode::NOT_FOUND);
            }
            tracing::warn!(endpoint, "CX ID upstream route returned 404");
            return Err(StatusCode::BAD_GATEWAY);
        }
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            tracing::warn!(endpoint, %status, "CX ID upstream rejected service authentication");
            return Err(StatusCode::BAD_GATEWAY);
        }
        if !status.is_success() {
            return Err(status);
        }
        response
            .json()
            .await
            .map(Json)
            .map_err(|_| StatusCode::BAD_GATEWAY)
    }
}
