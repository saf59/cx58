use crate::state::AppState;
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response, Sse, sse::Event},
};
use futures_util::{Stream, StreamExt, stream};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::{Duration, Instant};
use std::pin::Pin;
use tracing::{info, warn, error};

#[derive(Deserialize)]
pub struct PromptRequest {
    pub prompt: String,
}

pub async fn chat_stream_handler(
    State(state): State<AppState>,
    Json(req): Json<PromptRequest>,
) -> Result<impl IntoResponse, Response> {
    info!("Received streaming request for prompt: {}", req.prompt);

    let agent_url = state.oidc_client.config.agent_url.clone();
    let model_name = state.oidc_client.config.default_model.clone();

    #[derive(Serialize)]
    struct OllamaRequest {
        model: String,
        prompt: String,
        stream: bool,
    }

    let request_body = OllamaRequest {
        model: model_name,
        prompt: req.prompt,
        stream: true,
    };

    let client = reqwest::Client::new();

    let response = match client.post(&agent_url).json(&request_body).send().await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to send request to Ollama: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to connect to LLM: {}", e)})),
            ).into_response());
        }
    };

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Ollama returned an error: {}", error_text);
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": format!("LLM API Error: {}", error_text)})),
        ).into_response());
    }

    let byte_stream = response.bytes_stream();

    let sse_stream = byte_stream.flat_map(|chunk_result| {
        match chunk_result {
            Ok(chunk) => {
                let chunk_str = String::from_utf8_lossy(&chunk);
                let mut events: Vec<Result<Event, Infallible>> = Vec::new();
                for line in chunk_str.lines() {
                    if line.trim().is_empty() { continue; }
                    if let Ok(ollama_resp) = serde_json::from_str::<serde_json::Value>(line) {
                        if ollama_resp.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                            info!("LLM stream finished.");
                            continue;
                        }
                        if let Some(response_text) = ollama_resp.get("response").and_then(|v| v.as_str()) {
                            events.push(Ok(Event::default().data(response_text)));
                        }
                    }
                }
                Box::pin(stream::iter(events)) as Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>
            }
            Err(e) => {
                error!("Stream error: {}", e);
                Box::pin(stream::once(async move {
                    Ok::<Event, Infallible>(Event::default().event("error").data(format!("Stream error: {}", e)))
                })) as Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>
            }
        }
    });

    Ok(Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keepalive-text"),
    ))
}