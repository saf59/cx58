use crate::state::AppState;
use crate::auth::SESSION_ID;
use axum::{
    extract::State,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use reqwest::{Client, StatusCode};
use tower_cookies::Cookies;

// Stop handler - extracts session_id from cookie
pub async fn stop_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> impl IntoResponse {
    // Extract session_id from cookie
    let session_id = match jar.get(SESSION_ID) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            tracing::error!("No session_id in cookie");
            return (StatusCode::UNAUTHORIZED, "No session to exit");
        }
    };

    tracing::info!("Stop request for session: {}", session_id);

    let mut sessions = state.chat_sessions.lock().await;

    if let Some(chat_session) = sessions.get_mut(&session_id) {
        // Send local cancel signal
        let _ = chat_session.cancel_tx.send(true);

        // Cancel on agent if active request exists
        if let Some(request_id) = &chat_session.current_request_id.read().await.clone() {
            let agent_api_url = state.oidc_client.config.chat_config.agent_api_url.clone();
            let client = state.async_http_client.clone();
            cancel_agent_request(request_id, agent_api_url, client);
            //let mut req_id = chat_session.current_request_id.write().await;
            //*req_id = None;
        }

        (StatusCode::OK, "stopped")
    } else {
        tracing::warn!("Session {} not found", session_id);
        (StatusCode::NOT_FOUND, "session not found")
    }
}

pub fn cancel_agent_request(request_id: &String, agent_api_url: String, client: Client) {
    tracing::info!("Cancelling agent request: {}", request_id);

    let cancel_url = format!(
        "{}/agent/chat/cancel/{}",
        agent_api_url,
        request_id
    );

    let request_id = request_id.clone();

    tokio::spawn(async move {
        match client.delete(&cancel_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Agent request {} cancelled", request_id);
            }
            Ok(resp) => {
                tracing::warn!("Agent cancel status: {} on {}", resp.status(), &cancel_url);
            }
            Err(e) => {
                tracing::error!("Failed to cancel on agent: {}", e);
            }
        }
    });
}

