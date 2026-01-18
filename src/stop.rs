use crate::state::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
};
use reqwest::StatusCode;
use tower_cookies::Cookies;

// Stop handler - extracts session_id from cookie
pub async fn stop_handler(
    State(state): State<AppState>,
    cookies: Cookies,
) -> impl IntoResponse {
    // Extract session_id from cookie
    let session_id = match cookies.get("chat_session") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            tracing::warn!("No session_id in cookie");
            return (StatusCode::UNAUTHORIZED, "No session");
        }
    };

    tracing::info!("Stop request for session: {}", session_id);

    let mut sessions = state.chat_sessions.lock().await;

    if let Some(chat_session) = sessions.get_mut(&session_id) {
        // Send local cancel signal
        let _ = chat_session.cancel_tx.send(true);

        // Cancel on agent if active request exists
        if let Some(request_id) = &chat_session.current_request_id.read().await.clone() {
            tracing::info!("Cancelling agent request: {}", request_id);

            let cancel_url = format!(
                "{}/agent/chat/cancel/{}",
                state.oidc_client.config.chat_config.agent_api_url,
                request_id
            );

            let client = state.async_http_client.clone();
            let request_id = request_id.clone();

            tokio::spawn(async move {
                match client.delete(&cancel_url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::info!("Agent request {} cancelled", request_id);
                    }
                    Ok(resp) => {
                        tracing::warn!("Agent cancel status: {}", resp.status());
                    }
                    Err(e) => {
                        tracing::error!("Failed to cancel on agent: {}", e);
                    }
                }
            });

            let mut req_id = chat_session.current_request_id.write().await;
            *req_id = None;
        }

        (StatusCode::OK, "stopped")
    } else {
        tracing::warn!("Session {} not found", session_id);
        (StatusCode::NOT_FOUND, "session not found")
    }
}

