use crate::{
    auth_ssr::authenticated_operator,
    hmac::{build_hmac, build_operator_hmac},
    state::AppState,
};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use reqwest::{Method, RequestBuilder};
use serde_json::Value;
use uuid::Uuid;

pub async fn proxy_reports_handler(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
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
    Path(node_id): Path<Uuid>,
    cookies: CookieJar,
    Json(req): Json<Value>,
) -> Result<axum::response::Response, StatusCode> {
    let operator = authenticated_operator(&state, &cookies).await?;
    let target = format!("/agent/operator/reports/{node_id}");
    let body = serde_json::to_vec(&req).map_err(|_| StatusCode::BAD_REQUEST)?;
    let request = operator_request(
        &state,
        Method::PUT,
        &target,
        &operator,
        Some(&header::HeaderValue::from_static("application/json")),
        body.into(),
    )?;
    forward_mutation(request).await
}

pub async fn proxy_upload_image_handler(
    State(state): State<AppState>,
    Path(parent_id): Path<Uuid>,
    cookies: CookieJar,
    headers: HeaderMap,
    body: Bytes,
) -> Result<axum::response::Response, StatusCode> {
    let operator = authenticated_operator(&state, &cookies).await?;
    let target = format!("/agent/operator/images/upload/{parent_id}");
    let request = operator_request(
        &state,
        Method::POST,
        &target,
        &operator,
        headers.get(header::CONTENT_TYPE),
        body,
    )?;
    forward_mutation(request).await
}

pub async fn proxy_delete_image_handler(
    State(state): State<AppState>,
    Path(node_id): Path<Uuid>,
    cookies: CookieJar,
) -> Result<axum::response::Response, StatusCode> {
    let operator = authenticated_operator(&state, &cookies).await?;
    let target = format!("/agent/operator/images/{node_id}");
    let request = operator_request(
        &state,
        Method::DELETE,
        &target,
        &operator,
        None,
        Bytes::new(),
    )?;
    forward_mutation(request).await
}

fn operator_request(
    state: &AppState,
    method: Method,
    target: &str,
    operator: &str,
    content_type: Option<&header::HeaderValue>,
    body: Bytes,
) -> Result<RequestBuilder, StatusCode> {
    let config = &state.http_client.config.chat_config;
    let secret = config.agent_api_key.as_deref().unwrap_or_default();
    let (timestamp, signature) = build_operator_hmac(
        secret,
        method.as_str(),
        target,
        operator,
        content_type.map_or(&[][..], header::HeaderValue::as_bytes),
        &body,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let url = format!("{}{}", config.agent_api_url.trim_end_matches('/'), target);
    let mut request = state
        .async_http_client
        .request(method, url)
        .header("X-Timestamp", timestamp.to_string())
        .header("X-Signature", signature)
        .header("X-CX58-Operator", hex::encode(operator))
        .body(body);
    if let Some(content_type) = content_type {
        request = request.header(header::CONTENT_TYPE, content_type);
    }
    Ok(request)
}

async fn forward_mutation(request: RequestBuilder) -> Result<axum::response::Response, StatusCode> {
    let response = request.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    if response.status() == StatusCode::UNAUTHORIZED {
        tracing::warn!("Agent rejected report mutation service authentication");
        return Err(StatusCode::BAD_GATEWAY);
    }
    Ok(forward_empty_or_json_response(response).await)
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
