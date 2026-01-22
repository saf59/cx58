use crate::auth::SESSION_ID;
use crate::chunk_assembler::*;
use crate::events::*;
use crate::hmac::build_hmac;
use crate::state::AppState;
use crate::state::ChatSession;
use async_stream::stream;
use axum::{
    extract::State,
    response::{IntoResponse, Response, Sse, sse::Event},
};
use axum_extra::extract::CookieJar;
use futures::StreamExt;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
#[allow(unused_imports)]
use tracing::{error, info, warn};

#[derive(Debug, Deserialize, Serialize)]
pub struct PromptRequest {
    pub message: String,
    pub user_id: String,
    pub chat_id: String,
    pub language: String,
    pub object_id: Option<String>,
    pub prev_leaf: Option<String>,
    pub next_leaf: Option<String>,
}

pub async fn chat_stream_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    axum::Json(req): axum::Json<PromptRequest>,
) -> Result<impl IntoResponse, Response> {
    info!("Received streaming request for prompt: {}", &req.message);
    let session_id = match jar.get(SESSION_ID) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            tracing::error!("No session_id in cookie");
            return Err((StatusCode::UNAUTHORIZED, "No session to stop").into_response());
        }
    };
    let _chat_session = {
        let mut guard = state.chat_sessions.lock().await;
        guard
            .entry(session_id.clone())
            .or_insert_with(|| {
                Arc::new(ChatSession {
                    current_request_id: None.into(),
                })
            })
            .clone()
    };
    let chat_config = state.oidc_client.config.chat_config.clone();
    let start_at = Instant::now();
    let max_duration = Duration::from_secs(chat_config.max_duration_sec);
    let max_tokens: usize = chat_config.max_chat_tokens;
    let mut token_counter: usize = 0;
    let agent_url = format!("{}/agent/chat", &chat_config.agent_api_url);
    let agent_secret = chat_config.agent_api_key.clone().unwrap_or_default();

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
        .timeout(Duration::from_secs(300))
        .build()
        .unwrap();

    let sse_stream = stream! {
        let mut local_cache: Vec<String> = Vec::new();
        let mut retries = 0;
        let max_retries = 3;

        loop {
            info!("Sending request to agent (attempt {})", retries + 1);

            //let llm_req = client.post(&agent_url).json(&req);

            let req_bytes = serde_json::to_vec(&req).unwrap_or_default();
            let llm_req = match build_hmac(&agent_secret,&req_bytes) {
                Ok ((timestamp,signature)) => {
                    info!("Sending request with timestamp: {}", &timestamp);
                    info!("Signature: {}", &signature);
                    client.post(&agent_url)
                        .header("X-Timestamp", timestamp.to_string())
                        .header("X-Signature", signature)
                        .header("Content-Type", "application/json")
                        .json(&req)
                }
                Err(e) => {
                        tracing::warn!("Transport error: {:#?}", e);
                        break;
                }
            };



            let response_result = llm_req.send().await;
            info!("Response received from agent: {:?}", response_result.as_ref().map(|r| r.status()));

            let mut byte_stream = match response_result {
                Ok(res) if res.status().is_success() => {
                    res.bytes_stream()
                },
                Ok(res) => {
                    tracing::error!("Agent error response: {:#?}", res);
                    let status = res.status();
                    let text = res.text().await.unwrap_or_default();
                    tracing::error!("Agent error body: {}", text);
                    yield Ok(Event::default().event("error").data(format!("LLM error [{}]: {}", status, text)));
                    break;
                }
                Err(e) => {
                    tracing::warn!("Transport error: {:#?}", e);
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
                for cached in local_cache.iter() {
                    yield Ok(Event::default().event("replay").data(cached.clone()));
                }
            }

            let mut assembler = ChunkAssembler::new();
            let mut buffer = String::new();
            let mut chunk_count = 0;

            let finish_reason = 'outer: loop {
                tokio::select! {
                    _ = tokio::time::sleep_until((start_at + max_duration).into()) => {
                        info!("Stream timeout");
                        break 'outer FinishReason::Timeout;
                    }
                    maybe_chunk = byte_stream.next() => {
                        let bytes_chunk = match maybe_chunk {
                            Some(Ok(b)) => {
                                chunk_count += 1;
                                if chunk_count % 10 == 0 {
                                    info!("Received {} chunks so far", chunk_count);
                                }
                                b
                            },
                            Some(Err(e)) => {
                                tracing::warn!("Stream error: {:#?}", e);
                                yield Err(std::io::Error::other(e.to_string()));
                                break 'outer FinishReason::TransportError;
                            }
                            None => {
                                info!("Stream ended normally after {} chunks", chunk_count);
                                break 'outer FinishReason::Complete;
                            }
                        };

                        let text = String::from_utf8_lossy(&bytes_chunk);
                        buffer.push_str(&text);

                        // Parse SSE lines
                        while let Some(line_end) = buffer.find("\n\n") {
                            let sse_block = buffer[..line_end].to_string();
                            buffer = buffer[line_end + 2..].to_string();

                            // Extract data: from SSE
                            for line in sse_block.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {

                                    match serde_json::from_str::<StreamEvent>(data) {
                                        Ok(event) => {
                                            info!("Parsed StreamEvent: {:?}", event);
                                            let request_id = match &event {
                                                            StreamEvent::Started { request_id, .. }
                                                            | StreamEvent::TextChunk { request_id, .. }
                                                            | StreamEvent::CoordinatorThinking { request_id, .. }
                                                            | StreamEvent::ObjectChunk { request_id, .. }
                                                            | StreamEvent::DocumentChunk { request_id, .. }
                                                            | StreamEvent::DescriptionChunk { request_id, .. }
                                                            | StreamEvent::ComparisonChunk { request_id, .. }
                                                            | StreamEvent::Completed { request_id, .. }
                                                            | StreamEvent::Error { request_id, .. }
                                                            | StreamEvent::Cancelled { request_id, .. } => request_id,
                                                        };

                                            {
                                                    let mut sessions = state.chat_sessions.lock().await;
                                                    if let Some(session) = sessions.get_mut(&session_id) {
                                                         let mut req_id = session.current_request_id.write().await;
                                                        *req_id = Some(request_id.clone());
                                                    }
                                            }

                                            match event {
                                                StreamEvent::TextChunk { chunk, .. } => {

                                                    token_counter += chunk.split_whitespace().count();
                                                    local_cache.push(chunk.clone());
                                                    yield Ok(Event::default().data(chunk));

                                                    if token_counter >= max_tokens {
                                                        break 'outer FinishReason::MaxTokens;
                                                    }
                                                }
                                                StreamEvent::Started { request_id, .. } => {
                                                    tracing::debug!("Started request: {:?}", request_id);
                                                    yield Ok(Event::default().event("started").data(data));
                                                }
                                                StreamEvent::CoordinatorThinking { message, .. } => {
                                                    yield Ok(Event::default().event("coordinator_thinking").data(message));
                                                }
                                                StreamEvent::ObjectChunk { data: obj_data, .. } => {
                                                    let data_str = serde_json::to_string(&obj_data).unwrap_or_default();
                                                    yield Ok(Event::default().event("object_chunk").data(data_str));
                                                }
                                                StreamEvent::DocumentChunk { data: doc_data, .. } => {
                                                    let data_str = serde_json::to_string(&doc_data).unwrap_or_default();
                                                    yield Ok(Event::default().event("document_chunk").data(data_str));
                                                }
                                                StreamEvent::DescriptionChunk { data: desc_data, .. } => {
                                                    let data_str = serde_json::to_string(&desc_data).unwrap_or_default();
                                                    yield Ok(Event::default().event("description_chunk").data(data_str));
                                                }
                                                StreamEvent::ComparisonChunk { data: comp_data, .. } => {
                                                    let data_str = serde_json::to_string(&comp_data).unwrap_or_default();
                                                    yield Ok(Event::default().event("comparison_chunk").data(data_str));
                                                }
                                                StreamEvent::Completed { final_result, .. } => {
                                                    info!("Stream completed");
                                                    yield Ok(Event::default().event("completed").data(final_result));
                                                    break 'outer FinishReason::Complete;
                                                }
                                                StreamEvent::Error { error, .. } => {
                                                    tracing::error!("Agent error: {}", error);
                                                    yield Ok(Event::default().event("error").data(error));
                                                    break 'outer FinishReason::TransportError;
                                                }
                                                StreamEvent::Cancelled { reason, .. } => {
                                                    warn!("Stream cancelled: {}", reason);
                                                    yield Ok(Event::default().event("cancelled").data(reason));
                                                    break 'outer FinishReason::Stopped;
                                                }
                                            }
                                        }
                                        Err(parse_err) => {
                                            // If not StreamEvent, process as legacy format
                                            tracing::debug!("Not a StreamEvent ({}), using legacy parsing", parse_err);
                                            let chunks = assembler.push_sse_line(data);
                                            for chunk in chunks {
                                                match chunk {
                                                    UiChunk::Text(text) => {
                                                        token_counter += text.split_whitespace().count();
                                                        local_cache.push(text.clone());
                                                        yield Ok(Event::default().data(text));

                                                        if token_counter >= max_tokens {
                                                            break 'outer FinishReason::MaxTokens;
                                                        }
                                                    }
                                                    UiChunk::Markdown(md) => {
                                                        token_counter += md.split_whitespace().count();
                                                        local_cache.push(md.clone());
                                                        yield Ok(Event::default().data(md));

                                                        if token_counter >= max_tokens {
                                                            break 'outer FinishReason::MaxTokens;
                                                        }
                                                    }
                                                    UiChunk::Json(val) => {
                                                        let data_str = serde_json::to_string(&val).unwrap_or_default();
                                                        yield Ok(Event::default().event("json").data(data_str));
                                                        break 'outer FinishReason::Complete;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };

            info!("Stream finished with reason: {:?}", finish_reason);

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
