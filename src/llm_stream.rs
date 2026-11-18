use crate::auth::SESSION_ID;
use crate::chunk_assembler::*;
use crate::components::chat_data::ContextRequest;
use crate::events::*;
use crate::hmac::build_hmac;
use crate::state::AppState;
use crate::state::ChatSession;
use crate::stats::format_stats_table;
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
use tracing::{debug, error, info, warn};

#[derive(Debug, PartialEq, Eq)]
enum FinishReason {
    Complete,
    StreamEndedEarly,
    AgentError,
    Stopped,
    Timeout,
    MaxTokens,
    TransportError,
}

fn terminal_sse_event(reason: &FinishReason) -> Option<(&'static str, &'static str)> {
    match reason {
        FinishReason::Complete => Some(("on_complete", "ok")),
        FinishReason::StreamEndedEarly => Some(("on_stop", "stream_ended")),
        FinishReason::Stopped => Some(("on_stop", "by_user")),
        FinishReason::Timeout => Some(("on_stop", "timeout")),
        FinishReason::MaxTokens => Some(("on_stop", "max_tokens")),
        FinishReason::AgentError | FinishReason::TransportError => None,
    }
}

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

fn sse_block_end(bytes: &[u8]) -> Option<(usize, usize)> {
    bytes.iter().enumerate().find_map(|(index, byte)| {
        if *byte != b'\n' {
            return None;
        }
        match &bytes[index + 1..] {
            [b'\n', ..] => Some((index, 2)),
            [b'\r', b'\n', ..] => Some((index, 3)),
            _ => None,
        }
    })
}

pub async fn chat_stream_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    axum::Json(req): axum::Json<PromptRequest>,
) -> Result<impl IntoResponse, Response> {
    debug!("Received streaming request for prompt: {}", &req.message);
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
    let chat_config = state.http_client.config.chat_config.clone();
    let start_at = Instant::now();
    let max_duration = Duration::from_secs(chat_config.max_duration_sec);
    let max_tokens: usize = chat_config.max_chat_tokens;
    let mut token_counter: usize = 0;
    let agent_url = format!("{}/agent/chat", &chat_config.agent_api_url);
    let agent_secret = chat_config.agent_api_key.clone().unwrap_or_default();

    let client = state.async_http_client.clone();

    let sse_stream = stream! {
            let mut retries = 0;
            let max_retries = state.agent_max_retries;

        loop {
            debug!("Sending request to agent (attempt {})", retries + 1);
            let mut local_cache: Vec<String> = Vec::new();

            let req_bytes = match serde_json::to_vec(&req) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to serialize PromptRequest: {}", e);
                    yield Ok(Event::default().event("error")
                        .data(format!("transport-error|{}", e)));
                    break;
                }
            };

            let llm_req = match build_hmac(&agent_secret,&req_bytes) {
                Ok ((timestamp,signature)) => {
                    debug!("Sending request with timestamp: {} and signature: {}", &timestamp, &signature);
                    client.post(&agent_url)
                        .header("X-Timestamp", timestamp.to_string())
                        .header("X-Signature", signature)
                        .header("Content-Type", "application/json")
                        .body(req_bytes)
                }
                Err(e) => {
                    tracing::warn!("Transport error: {:#?}", e);
                    yield Ok(Event::default().event("error")
                        .data(format!("transport-error|{}", e)));
                    break;
                }
            };

            let response_result = llm_req.send().await;
            debug!("Response received from agent: {:?}", response_result.as_ref().map(|r| r.status()));

            let mut byte_stream = match response_result {
                Ok(res) if res.status().is_success() => {
                    res.bytes_stream()
                },
                Ok(res) => {
                    let status = res.status().as_u16().to_string();
                    let text = res.text().await.unwrap_or_default();
                    tracing::error!("Agent error: {} — {}", status, text);
                    yield Ok(Event::default().event("error")
                        .data(format!("llm-error|{}|{}", status, text)));
                    break;
                }
            Err(e) => {
                tracing::warn!("Transport error: {:#?}", e);
                yield Ok(Event::default().event("error")
                    .data(format!("transport-error|{}", e)));
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

        let mut buffer = Vec::new();
        let finish_reason = 'outer: loop {
            tokio::select! {
                _ = tokio::time::sleep_until((start_at + max_duration).into()) => {
                    warn!("Stream timeout");
                    break 'outer FinishReason::Timeout;
                }
                maybe_chunk = byte_stream.next() => {
                    let bytes_chunk = match maybe_chunk {
                        Some(Ok(bytes)) => bytes,
                        Some(Err(e)) => {
                            tracing::warn!("Stream error: {:#?}", e);
                            yield Ok::<Event, std::io::Error>(Event::default().event("error")
                                .data(format!("transport-error|{}", e)));
                            break 'outer FinishReason::TransportError;
                        }
                        None => {
                            break 'outer FinishReason::StreamEndedEarly;
                        }
                    };

                    buffer.extend_from_slice(&bytes_chunk);

                    // Decode only complete SSE blocks to preserve split UTF-8 and CRLF.
                    while let Some((end, delimiter_len)) = sse_block_end(&buffer) {
                        let sse_block = String::from_utf8_lossy(&buffer[..end]).into_owned();
                        buffer.drain(..end + delimiter_len);

                        // Extract data: from SSE
                        for line in sse_block.lines() {
                            if let Some(_evt) = line.strip_prefix("event: ") {
                                continue;
                            }
                            if let Some(data) = line.strip_prefix("data: ") {

                                match serde_json::from_str::<StreamEvent>(data) {
                                    Ok(event) => {
                                        let request_id = event.request_id();
                                        {
                                            let mut sessions = state.chat_sessions.lock().await;
                                            if let Some(session) = sessions.get_mut(&session_id) {
                                                 let mut req_id = session.current_request_id.write().await;
                                                *req_id = Some(request_id.to_string());
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
                                            StreamEvent::Progress { message, .. } => {
                                                yield Ok(Event::default().event("progress").data(message));
                                            }
                                            StreamEvent::ObjectTree { data: obj_data, .. } => {
                                                let data_str = serde_json::to_string(&obj_data).unwrap_or_default();
                                                yield Ok(Event::default().event("object").data(data_str));
                                            }
                                            StreamEvent::ReportList { data: doc_data, .. } => {
                                                let data_str = serde_json::to_string(&doc_data).unwrap_or_default();
                                                yield Ok(Event::default().event("report_list").data(data_str));
                                            }
                                            StreamEvent::Description { data: desc_data, .. } => {
                                                let data_str = serde_json::to_string(&desc_data).unwrap_or_default();
                                                yield Ok(Event::default().event("description").data(data_str));
                                            }
                                            StreamEvent::Comparison { data: comp_data, .. } => {
                                                let data_str = serde_json::to_string(&comp_data).unwrap_or_default();
                                                yield Ok(Event::default().event("comparison").data(data_str));
                                            }
                                            StreamEvent::ContextRequest { prompt, suggestions, .. } => {
                                                 let data = serde_json::to_string(&ContextRequest {
                                                    prompt,
                                                    suggestions,
                                                    }).unwrap_or_default();
                                                debug!("ContextRequest event with data: {}", data);
                                                yield Ok(Event::default().event("context_request").data(data));
                                            }
                                            StreamEvent::Completed { total_time_ms, stats,.. } => {
                                                info!("{}", format_stats_table(total_time_ms, &stats));
                                                break 'outer FinishReason::Complete;
                                            }
                                            StreamEvent::Error { error, .. } => {
                                                tracing::error!("Agent error: {}", error);
                                                let message = error.lines().collect::<Vec<_>>().join(" ");
                                                yield Ok(Event::default().event("error").data(message));
                                                break 'outer FinishReason::AgentError;
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

        if let Some((event, data)) = terminal_sse_event(&finish_reason) {
            yield Ok(Event::default().event(event).data(data));
            break;
        }

        match finish_reason {
            FinishReason::AgentError => break,
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
            _ => unreachable!("non-transport finish reasons return above"),
        }
    }

        let mut guard = state.chat_sessions.lock().await;
        guard.remove(&session_id);
        debug!("GC: ChatSession {} removed", session_id);
    };

    Ok(Sse::new(sse_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text(": keepalive"),
    ))
}

#[cfg(test)]
mod finish_reason_tests {
    use super::{FinishReason, sse_block_end, terminal_sse_event};

    #[test]
    fn terminal_agent_error_has_no_second_stop_event() {
        assert_eq!(terminal_sse_event(&FinishReason::AgentError), None);
    }

    #[test]
    fn waits_for_the_complete_utf8_sse_block_and_line_ending() {
        for ending in ["\n\n", "\r\n\r\n"] {
            let event = format!("data: Küche – Схід{ending}");
            let bytes = event.as_bytes();
            for split in 0..bytes.len() {
                assert!(sse_block_end(&bytes[..split]).is_none());
            }
            let (end, delimiter) = sse_block_end(bytes).unwrap();
            assert_eq!(end + delimiter, bytes.len());
            assert_eq!(
                std::str::from_utf8(&bytes[..end]).unwrap().trim_end(),
                "data: Küche – Схід"
            );
        }
    }

    #[test]
    fn early_eof_is_not_reported_as_success() {
        assert_eq!(
            terminal_sse_event(&FinishReason::StreamEndedEarly),
            Some(("on_stop", "stream_ended"))
        );
        assert_eq!(
            terminal_sse_event(&FinishReason::Complete),
            Some(("on_complete", "ok"))
        );
    }
}
