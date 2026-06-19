use crate::{hmac::build_hmac, state::AppState};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde_json::Value;

pub async fn proxy_reports_handler(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> impl IntoResponse {
    let chat_config = &state.http_client.config.chat_config;
    let agent_url = format!("{}/agent/reports/{}", chat_config.agent_api_url, node_id);
    let agent_secret = chat_config.agent_api_key.clone().unwrap_or_default();

    let (timestamp, signature) = match build_hmac(&agent_secret, &[]) {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to sign request: {}", e)
                })),
            )
                .into_response();
        }
    };

    match state
        .async_http_client
        .post(&agent_url)
        .header("X-Timestamp", timestamp.to_string())
        .header("X-Signature", signature)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(response) => forward_json_response(response).await,
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to fetch reports from agent: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn proxy_update_report_handler(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let chat_config = &state.http_client.config.chat_config;
    let agent_url = format!("{}/agent/reports/{}", chat_config.agent_api_url, node_id);
    let agent_secret = chat_config.agent_api_key.clone().unwrap_or_default();

    let body = match serde_json::to_vec(&req) {
        Ok(body) => body,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Failed to serialize request: {}", e)
                })),
            )
                .into_response();
        }
    };

    let (timestamp, signature) = match build_hmac(&agent_secret, &body) {
        Ok(value) => value,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to sign request: {}", e)
                })),
            )
                .into_response();
        }
    };

    match state
        .async_http_client
        .put(&agent_url)
        .header("X-Timestamp", timestamp.to_string())
        .header("X-Signature", signature)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(response) => forward_empty_or_json_response(response).await,
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to update report through agent: {}", e)
            })),
        )
            .into_response(),
    }
}
pub async fn proxy_upload_image_handler(
    State(state): State<AppState>,
    Path(parent_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let chat_config = &state.http_client.config.chat_config;
    let agent_url = format!(
        "{}/agent/images/upload/{}",
        chat_config.agent_api_url, parent_id
    );

    let mut request = state.async_http_client.post(&agent_url).body(body);
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        request = request.header(header::CONTENT_TYPE, content_type.clone());
    }

    match request.send().await {
        Ok(response) => forward_json_response(response).await,
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to upload image through agent: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn proxy_delete_image_handler(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> impl IntoResponse {
    let chat_config = &state.http_client.config.chat_config;
    let agent_url = format!("{}/agent/images/{}", chat_config.agent_api_url, node_id);

    match state.async_http_client.delete(&agent_url).send().await {
        Ok(response) => {
            let status = response.status();
            if status == StatusCode::NO_CONTENT || status == StatusCode::OK {
                return status.into_response();
            }
            forward_json_response(response).await
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to delete image through agent: {}", e)
            })),
        )
            .into_response(),
    }
}

async fn forward_json_response(response: reqwest::Response) -> axum::response::Response {
    let status = response.status();
    match response.json::<Value>().await {
        Ok(json) => (status, Json(json)).into_response(),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to parse agent response: {}", e)
            })),
        )
            .into_response(),
    }
}

async fn forward_empty_or_json_response(response: reqwest::Response) -> axum::response::Response {
    let status = response.status();
    if status == StatusCode::NO_CONTENT || response.content_length() == Some(0) {
        return status.into_response();
    }
    forward_json_response(response).await
}
