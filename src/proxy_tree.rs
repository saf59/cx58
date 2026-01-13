use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use reqwest::Client;
use serde_json::Value;

/// Proxy handler for tree API
pub async fn proxy_tree_handler(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let agent_api_url = &state.oidc_client.config.chat_config.agent_api_url;
    println!("Proxying tree request for user_id: {}", user_id);
    // Extract query parameters (with_leafs)
    let with_leafs = headers
        .get("x-with-leafs")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let url = if with_leafs {
        format!("{}/api/agent/tree/{}?with_leafs=true", agent_api_url, user_id)
    } else {
        format!("{}/api/agent/tree/{}", agent_api_url, user_id)
    };

    // Forward request to backend
    let client = Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            match response.json::<Value>().await {
                Ok(json) => (status, Json(json)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to parse response: {}", e)
                    }))
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": format!("Failed to fetch from backend: {}", e)
            }))
        ).into_response(),
    }
}

// Add router entry in main.rs:
// .route("/api/proxy/tree/:user_id", get(proxy_tree_handler))