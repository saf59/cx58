use crate::state::AppState;
use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
pub struct StopRequest {
    pub session_id: String,
}

pub async fn stop_handler(
    State(state): State<AppState>,
    Json(req): Json<StopRequest>,
) -> impl IntoResponse {
    info!("Received stop signal!");
    let sessions = state.chat_sessions.lock().await;
    if let Some(chat_session) = sessions.get(&req.session_id) {
        let _ = chat_session.cancel_tx.send(true);
        info!("ChatSession {} stopped by user", req.session_id);
        (axum::http::StatusCode::OK, "stopped")
    } else {
        (axum::http::StatusCode::NOT_FOUND, "session not found")
    }
}
