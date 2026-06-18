use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSettings {
    pub user_id: String,
    pub vision_model: String,
    pub text_model: String,
    pub chat_model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub user_id: String,
    pub current: ModelSettings,
    pub defaults: ModelSettings,
    #[serde(default)]
    pub models: Vec<OllamaModelInfo>,
    pub capability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateModelsRequest {
    pub vision_model: Option<String>,
    pub text_model: Option<String>,
    pub chat_model: Option<String>,
    pub same: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelChange {
    pub role: String,
    pub model: String,
    pub applied: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateModelsResponse {
    pub user_id: String,
    pub current: ModelSettings,
    #[serde(default)]
    pub changes: Vec<ModelChange>,
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::UpdateModelsRequest;
    use crate::{hmac::build_hmac, state::AppState};
    use axum::{
        Json,
        extract::{Path, State},
        http::StatusCode,
        response::IntoResponse,
    };
    use serde_json::Value;

    pub async fn get_models_handler(
        State(state): State<AppState>,
        Path(user_id): Path<String>,
    ) -> impl IntoResponse {
        let chat_config = &state.http_client.config.chat_config;
        let agent_url = format!("{}/agent/models/{}", chat_config.agent_api_url, user_id);
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
            .get(&agent_url)
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
                    "error": format!("Failed to fetch models from agent: {}", e)
                })),
            )
                .into_response(),
        }
    }

    pub async fn update_models_handler(
        State(state): State<AppState>,
        Path(user_id): Path<String>,
        Json(req): Json<UpdateModelsRequest>,
    ) -> impl IntoResponse {
        let chat_config = &state.http_client.config.chat_config;
        let agent_url = format!("{}/agent/models/{}", chat_config.agent_api_url, user_id);
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
            Ok(response) => forward_json_response(response).await,
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": format!("Failed to update models through agent: {}", e)
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
}

#[cfg(feature = "ssr")]
pub use ssr::{get_models_handler, update_models_handler};
