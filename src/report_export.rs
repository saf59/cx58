use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportRequest {
    #[serde(rename = "request")]
    pub text: String,
    pub language: String,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Json,
}

impl ExportFormat {
    pub fn filename(self) -> &'static str {
        match self {
            Self::Pdf => "cx58-report.pdf",
            Self::Json => "cx58-report.json",
        }
    }

    pub fn media_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Json => "application/json",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportSources {
    Description { report_id: Uuid },
    Comparison { previous_id: Uuid, current_id: Uuid },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerOutletCount {
    pub confirmed: u32,
    pub uncertain: bool,
}

impl PowerOutletCount {
    pub fn display(self, uncertain_label: &str) -> String {
        if self.uncertain {
            format!("{} ({uncertain_label})", self.confirmed)
        } else {
            self.confirmed.to_string()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerOutletComparison {
    pub previous: Option<PowerOutletCount>,
    pub current: Option<PowerOutletCount>,
}

impl PowerOutletComparison {
    pub fn display(self, uncertain_label: &str, unavailable: &str) -> String {
        let format_count = |count: Option<PowerOutletCount>| match count {
            Some(count) => count.display(uncertain_label),
            None => unavailable.to_string(),
        };
        format!(
            "{} → {}",
            format_count(self.previous),
            format_count(self.current)
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PowerOutletResult {
    Photo(PowerOutletCount),
    Comparison(PowerOutletComparison),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub description: String,
    pub windows: Option<String>,
    pub doors: Option<String>,
    pub radiators: Option<String>,
    pub openings: Option<String>,
    pub power_outlets: Option<PowerOutletResult>,
    pub model_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReportExport {
    #[serde(flatten)]
    pub request: ReportRequest,
    pub sources: ExportSources,
    pub result: ExportResult,
}

#[derive(Serialize, Deserialize)]
pub struct ExportDownload {
    pub format: ExportFormat,
    #[serde(flatten)]
    pub report: ReportExport,
}

#[cfg(feature = "ssr")]
pub mod proxy {
    use super::ExportDownload;
    use crate::{auth_ssr::authenticated_operator, hmac::build_hmac, state::AppState};
    use axum::{
        Json,
        body::Body,
        extract::State,
        http::{StatusCode, header},
        response::{IntoResponse, Response},
    };
    use axum_extra::extract::CookieJar;
    use std::time::Duration;

    pub async fn download(
        State(state): State<AppState>,
        cookies: CookieJar,
        Json(request): Json<ExportDownload>,
    ) -> Response {
        (
            [(header::CACHE_CONTROL, "no-store")],
            forward(&state, &cookies, request).await,
        )
            .into_response()
    }

    async fn forward(
        state: &AppState,
        cookies: &CookieJar,
        request: ExportDownload,
    ) -> Result<Response, StatusCode> {
        let user_id = authenticated_operator(state, cookies).await?;
        #[derive(serde::Serialize)]
        struct AgentRequest {
            user_id: String,
            #[serde(flatten)]
            download: ExportDownload,
        }
        let format = request.format;
        let body = serde_json::to_vec(&AgentRequest {
            user_id,
            download: request,
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let config = &state.http_client.config.chat_config;
        let secret = config
            .agent_api_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let (timestamp, signature) =
            build_hmac(secret, &body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let mut upstream = state
            .async_http_client
            .post(format!("{}/agent/report-export", config.agent_api_url))
            .header("X-Timestamp", timestamp.to_string())
            .header("X-Signature", signature)
            .header(header::CONTENT_TYPE, "application/json")
            .timeout(Duration::from_secs(45))
            .body(body)
            .send()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        if !upstream.status().is_success() {
            if upstream.status() == StatusCode::UNAUTHORIZED {
                tracing::warn!("Report export upstream rejected service authentication");
                return Err(StatusCode::BAD_GATEWAY);
            }
            return Err(upstream.status());
        }
        if upstream
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            != Some(format.media_type())
        {
            return Err(StatusCode::BAD_GATEWAY);
        }
        const MAX_DOWNLOAD_BYTES: usize = 16 * 1024 * 1024;
        let mut bytes = Vec::new();
        while let Some(chunk) = upstream
            .chunk()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?
        {
            if chunk.len() > MAX_DOWNLOAD_BYTES - bytes.len() {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok((
            [
                (header::CONTENT_TYPE, format.media_type().to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", format.filename()),
                ),
            ],
            Body::from(bytes),
        )
            .into_response())
    }
}
