use crate::{hmac::build_hmac, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::Value;

/// Proxy handler for tree API
pub async fn proxy_tree_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let chat_config = &state.http_client.config.chat_config;
    let agent_api_url = &chat_config.agent_api_url;
    let agent_secret = chat_config.agent_api_key.clone().unwrap_or_default();
    // Extract query parameters (with_leafs)
    let with_leafs = headers
        .get("x-with-leafs")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let url = if with_leafs {
        format!("{}/agent/tree/{}?with_leafs=true", agent_api_url, user_id)
    } else {
        format!("{}/agent/tree/{}", agent_api_url, user_id)
    };

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
        .get(&url)
        .header("X-Timestamp", timestamp.to_string())
        .header("X-Signature", signature)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json) => (status, Json(json)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to parse response: {}", e)
                    })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to fetch from backend: {}", e)
            })),
        )
            .into_response(),
    }
}

// Add router entry in main.rs:
// .route("/api/proxy/tree/:user_id", get(proxy_tree_handler))
