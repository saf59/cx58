use crate::chunk_assembler::*;
use crate::state::{AppState, ChatSession};
use async_stream::stream;
use axum::{
    extract::State,
    response::{IntoResponse, Response, Sse, sse::Event},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;
#[allow(unused_imports)]
use tracing::{error, info, warn};

#[derive(Deserialize)]
pub struct PromptRequest {
    pub prompt: String,
    pub session_id: String,
}

pub async fn chat_stream_handler(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<PromptRequest>,
) -> Result<impl IntoResponse, Response> {
    info!("Received streaming request for prompt: {}", req.prompt);
    let session_id = req.session_id;

    let chat_session = {
        let mut guard = state.chat_sessions.lock().await;
        guard
            .entry(session_id.clone())
            .or_insert_with(|| {
                let (tx, rx) = watch::channel(false);
                Arc::new(ChatSession {
                    cancel_tx: tx,
                    cancel_rx: rx,
                    cache: tokio::sync::Mutex::new(Vec::new()),
                })
            })
            .clone()
    };
    let chat_config = state.oidc_client.config.chat_config.clone();

    let start_at = Instant::now();
    let max_duration = Duration::from_secs(chat_config.max_duration_sec);
    let max_tokens: usize = chat_config.max_chat_tokens;
    let mut token_counter: usize = 0;
    let agent_url = format!("{}/api/agent/chat", &chat_config.agent_api_url);
    let model_name = chat_config.agent_model.clone();

    #[derive(Debug, Serialize)]
    struct AgentRequest {
        model: String,
        prompt: String,
        stream: bool,
        format: serde_json::Value,
    }
    let format = serde_json::from_str(
        r#"{
            "type": "object",
            "properties": {
                "text": {
                    "type": "string"
                },
                "objects": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "item": {
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string"
                                },
                                "url": {
                                    "type": "string"
                                }
                            }
                        }
                    }
                }
            },
            "required": [
                "text"
            ]
    }"#,
    )
    .unwrap();

    let request_body = AgentRequest {
        model: model_name,
        prompt: req.prompt.clone(),
        stream: true,
        format,
    };

    #[derive(Debug)]
    enum FinishReason {
        Complete,
        Stopped,
        Timeout,
        MaxTokens,
        TransportError,
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let sse_stream = stream! {
        let mut stop_flag = chat_session.cancel_rx.clone();
        let mut retries = 0;
        let max_retries = 3;
        //tracing::info!("{:#?}",&request_body);
        loop {
            let mut llm_req = client.post(&agent_url).json(&request_body);
            if let Some(ref agent_api_key) = chat_config.agent_api_key {
                llm_req = llm_req.header("Authorization", format!("Basic {}", agent_api_key));
            }
            let response_result = llm_req.send().await;

            let mut byte_stream = match response_result {
                Ok(res) if res.status().is_success() => res.bytes_stream(),
                Ok(res) => {
                    tracing::error!("{:#?}",res);
                    let text = res.text().await.unwrap_or_default();
                    yield Ok(Event::default().event("error").data(format!("LLM error: {}", text)));
                    break;
                }
                Err(e) => {
                    tracing::warn!("{:#?}",e);
                    yield Ok(Event::default().event("error").data(format!("Transport error: {}", e)));
                    if retries < max_retries {
                        retries += 1;
                        tokio::time::sleep(Duration::from_secs(1 << retries)).await;
                        continue;
                    } else {
                        break;
                    }
                }
            };

            {
                let cache_guard = chat_session.cache.lock().await;
                for cached in cache_guard.iter() {
                    yield Ok(Event::default().event("replay").data(cached.clone()));
                }
            }

            let mut assembler = ChunkAssembler::new();

            let finish_reason = 'outer: loop {
                tokio::select! {
                    _ = stop_flag.changed() => {
                        break 'outer FinishReason::Stopped;
                    }
                    _ = tokio::time::sleep_until((start_at + max_duration).into()) => {
                        break 'outer FinishReason::Timeout;
                    }
                    maybe_chunk = byte_stream.next() => {
                        let bytes_chunk = match maybe_chunk {
                            Some(Ok(b)) => b,
                            Some(Err(e)) => {
                                tracing::warn!("{:#?}",e);
                                yield Err(std::io::Error::other(e.to_string()));
                                break 'outer FinishReason::TransportError;
                            }
                            None => break 'outer FinishReason::Complete,
                        };

                        let text = String::from_utf8_lossy(&bytes_chunk);
                        //tracing::info!("{:?}",text);
                        let chunks = assembler.push_sse_line(&text);

                        for chunk in chunks {
                            match chunk {
                                // Стриминг текста - отправляем сразу
                                UiChunk::Text(text) => {
                                    token_counter += text.split_whitespace().count();
                                    {
                                        let mut cache_guard = chat_session.cache.lock().await;
                                        cache_guard.push(text.clone());
                                    }
                                    yield Ok(Event::default().data(text));

                                    if token_counter >= max_tokens {
                                        break 'outer FinishReason::MaxTokens;
                                    }
                                }

                                // Markdown тоже стримим
                                UiChunk::Markdown(md) => {
                                    token_counter += md.split_whitespace().count();
                                    {
                                        let mut cache_guard = chat_session.cache.lock().await;
                                        cache_guard.push(md.clone());
                                    }
                                    yield Ok(Event::default().data(md));

                                    if token_counter >= max_tokens {
                                        break 'outer FinishReason::MaxTokens;
                                    }
                                }

                                // Финальный JSON - отправляем с событием "json"
                                UiChunk::Json(val) => {
                                    let data_str = serde_json::to_string(&val).unwrap_or_default();
                                    yield Ok(Event::default().event("json").data(data_str));
                                    break 'outer FinishReason::Complete;
                                }
                            }
                        }
                    }
                }
            };

            match finish_reason {
                FinishReason::Complete => {
                    yield Ok(Event::default().event("on_complete").data("ok"));
                    break;
                }
                FinishReason::Stopped => {
                    yield Ok(Event::default().event("on_stop").data("by_user"));
                    break;
                }
                FinishReason::Timeout => {
                    yield Ok(Event::default().event("on_stop").data("timeout"));
                    break;
                }
                FinishReason::MaxTokens => {
                    yield Ok(Event::default().event("on_stop").data("max_tokens"));
                    break;
                }
                FinishReason::TransportError => {
                    retries += 1;
                    if retries <= max_retries {
                        tokio::time::sleep(Duration::from_secs(1 << retries)).await;
                        continue;
                    } else {
                        yield Ok(Event::default().event("on_stop").data("transport_error"));
                        break;
                    }
                }
            }
        }

        let mut guard = state.chat_sessions.lock().await;
        guard.remove(&session_id);
        info!("GC: ChatSession {} removed", session_id);
    };

    Ok(Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive-text"),
    ))
}
