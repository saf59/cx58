use crate::app::shell;
use crate::auth::*;
use crate::auth_ssr::*;
use crate::config::AppConfig;
use crate::state::{AppState, ChatSession};
use axum::{
    Json,
    extract::{FromRef, OriginalUri, Query, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use cookie::time::Duration as CookieDuration;
use http::HeaderMap;
use leptos::config::LeptosOptions;
use leptos::context::provide_context;
use leptos::serde_json;
use leptos_axum::handle_server_fns_with_context;
use oauth2::basic::{BasicErrorResponseType, BasicRevocationErrorResponse};
use oauth2::{
    AccessToken, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken,
    Scope, StandardErrorResponse, StandardRevocableToken,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreGenderClaim, CoreIdTokenClaims,
    CoreIdTokenVerifier, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
    CoreResponseType, CoreTokenIntrospectionResponse, CoreTokenResponse,
};
use openidconnect::{
    AuthenticationFlow, EmptyAdditionalClaims, IssuerUrl, Nonce, OAuth2TokenResponse,
};
use serde::{Deserialize, Serialize, de::Error};
use serde_json::Value;
use serde_urlencoded::de::Error as UrlError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
#[allow(unused_imports)]
use tracing::{debug, info, warn};
use uuid::Uuid;

const SSO_PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(5);
const SSO_TOKEN_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(15);
const SSO_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize)]
pub struct ReadinessStatus {
    status: &'static str,
    ui: &'static str,
    sso: &'static str,
    request_id: String,
    elapsed_ms: u128,
}

fn readiness_status(
    sso_available: bool,
    request_id: Uuid,
    elapsed_ms: u128,
) -> (StatusCode, ReadinessStatus) {
    let (http_status, status, sso) = if sso_available {
        (StatusCode::OK, "ready", "available")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded", "unavailable")
    };
    (
        http_status,
        ReadinessStatus {
            status,
            ui: "available",
            sso,
            request_id: request_id.to_string(),
            elapsed_ms,
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthLanguage {
    En,
    De,
}

impl AuthLanguage {
    fn from_hint(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some(value) if value.starts_with("de") => Self::De,
            _ => Self::En,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
        }
    }
}

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    lang: Option<String>,
}

struct ValidatedIdTokenData {
    id_token: String,
    subject: String,
    expires_in: Duration,
    roles: HashSet<Role>,
    name: String,
    email: Option<String>,
}

fn auth_language(jar: &CookieJar, query_language: Option<&str>) -> AuthLanguage {
    let language_hint = query_language.or_else(|| jar.get("lf-lang").map(|cookie| cookie.value()));
    AuthLanguage::from_hint(language_hint)
}

fn oidc_discovery_url(issuer_url: &str) -> String {
    format!(
        "{}/.well-known/openid-configuration",
        issuer_url.trim_end_matches('/')
    )
}

fn sso_unavailable_response(
    language: AuthLanguage,
    request_id: &str,
    status: StatusCode,
) -> Response {
    let (title, message, retry, home) = match language {
        AuthLanguage::En => (
            "Sign-in service unavailable",
            "The single sign-on service is currently unavailable. Please try again.",
            "Repeat",
            "Home",
        ),
        AuthLanguage::De => (
            "Anmeldedienst nicht verfügbar",
            "Der Single-Sign-on-Dienst ist derzeit nicht erreichbar. Bitte versuchen Sie es erneut.",
            "Wiederholen",
            "Startseite",
        ),
    };
    auth_error_response(language, request_id, status, title, message, retry, home)
}

fn callback_invalid_response(
    language: AuthLanguage,
    request_id: &str,
    status: StatusCode,
) -> Response {
    let (title, message, retry, home) = match language {
        AuthLanguage::En => (
            "Sign-in could not be completed",
            "The sign-in request is no longer valid. Please start sign-in again.",
            "Start sign-in again",
            "Home",
        ),
        AuthLanguage::De => (
            "Anmeldung konnte nicht abgeschlossen werden",
            "Die Anmeldeanfrage ist nicht mehr gültig. Bitte starten Sie die Anmeldung erneut.",
            "Anmeldung erneut starten",
            "Startseite",
        ),
    };
    auth_error_response(language, request_id, status, title, message, retry, home)
}

#[allow(clippy::too_many_arguments)]
fn auth_error_response(
    language: AuthLanguage,
    request_id: &str,
    status: StatusCode,
    title: &str,
    message: &str,
    retry: &str,
    home: &str,
) -> Response {
    let html = format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    body {{ margin: 0; font-family: system-ui, -apple-system, system-ui; background: #f6f4f0; color: #444; text-align: center; }}
    main {{ box-sizing: border-box; display: flex; min-height: 100svh; width: 100%; padding: 2rem; flex-direction: column; justify-content: center; align-items: center; }}
    h1 {{ margin: 0 0 1em; font-size: 1.17em; }}
    p {{ margin: 0 0 1em; }}
    .actions {{ display: flex; gap: 0.75rem; flex-wrap: wrap; justify-content: center; }}
    a {{ display: inline-block; padding: 0 5px; color: #444; line-height: 28px; border-radius: 5px; border: none; text-decoration: none; }}
    a:hover {{ color: #40c0e7; background: #fff; }}
    small {{ display: block; margin-top: 1.5rem; color: #777; overflow-wrap: anywhere; }}
  </style>
</head>
<body>
  <main>
    <h1>{title}</h1>
    <p>{message}</p>
    <div class="actions">
      <a href="/login?lang={lang}">{retry}</a>
      <a class="secondary" href="/">{home}</a>
    </div>
    <small>Reference: {request_id}</small>
  </main>
</body>
</html>"#,
        lang = language.code(),
    );

    (status, Html(html)).into_response()
}

fn correlation_id(headers: &HeaderMap) -> String {
    ["x-request-id", "x-correlation-id"]
        .into_iter()
        .find_map(|name| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| {
                    !value.is_empty()
                        && value.len() <= 128
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.:".contains(&byte))
                })
                .map(str::to_string)
        })
        .unwrap_or_else(|| Uuid::now_v7().to_string())
}

fn validated_id_token_data(id_token: String, claims: &CoreIdTokenClaims) -> ValidatedIdTokenData {
    let expiry_system_time: SystemTime = claims.expiration().into();
    let expires_in = expiry_system_time
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);
    let claims_json = serde_json::to_value(claims).ok();

    ValidatedIdTokenData {
        id_token,
        subject: claims.subject().to_string(),
        expires_in,
        roles: claims_json
            .as_ref()
            .map(extract_roles_from_claims)
            .unwrap_or_default(),
        name: claims_json
            .as_ref()
            .map(extract_name_from_claims)
            .unwrap_or_else(|| claims.subject().to_string()),
        email: claims_json.as_ref().and_then(extract_email_from_claims),
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct ISPOidcClient {
    pub client: openidconnect::Client<
        EmptyAdditionalClaims,
        CoreAuthDisplay,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJsonWebKey,
        CoreAuthPrompt,
        StandardErrorResponse<BasicErrorResponseType>,
        CoreTokenResponse,
        CoreTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointMaybeSet,
        EndpointMaybeSet,
    >,
    pub config: AppConfig,
}

impl ISPOidcClient {
    pub async fn new(async_http_client: &reqwest::Client) -> anyhow::Result<Self> {
        let config = AppConfig::from_env().expect("Failed to load config");
        tracing::info!("issuer={:?}", &config.oidc_issuer_url);
        tracing::debug!(
            oidc_issuer_url = %config.oidc_issuer_url,
            oidc_redirect_uri = %config.oidc_redirect_uri,
            oidc_client_id = %config.oidc_client_id,
            app_is_prod = config.is_prod,
            "OIDC runtime config loaded"
        );
        let issuer = IssuerUrl::new(config.oidc_issuer_url.clone())?;
        let provider_metadata =
            CoreProviderMetadata::discover_async(issuer, async_http_client).await?;
        let client_id = ClientId::new(config.oidc_client_id.clone());
        let client_secret = Some(ClientSecret::new(config.oidc_client_secret.clone()));
        let redirect_uri = RedirectUrl::new(config.oidc_redirect_uri.clone())?;

        let inner = CoreClient::from_provider_metadata(provider_metadata, client_id, client_secret)
            .set_redirect_uri(redirect_uri);
        Ok(ISPOidcClient {
            client: inner,
            config,
        })
    }

    pub fn authorize_url(&self) -> (url::Url, CsrfToken, Nonce, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("roles".to_string())) // Add roles scope
            .set_pkce_challenge(pkce_challenge)
            .url();

        debug!(auth_url = %auth_url, "OIDC authorization URL generated");
        (auth_url, csrf_token, nonce, pkce_verifier)
    }

    pub async fn exchange_code(
        &self,
        code: AuthorizationCode,
        pkce_verifier: PkceCodeVerifier,
        async_http_client: &reqwest::Client,
    ) -> anyhow::Result<CoreTokenResponse> {
        let token_response = self
            .client
            .exchange_code(code)
            .expect("Client not properly configured")
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;
        Ok(token_response)
    }

    pub async fn exchange_refresh_token(
        &self,
        refresh_token: &RefreshToken,
        async_http_client: &reqwest::Client,
    ) -> anyhow::Result<CoreTokenResponse> {
        let token_response = self
            .client
            .exchange_refresh_token(refresh_token)
            .expect("OIDC client misconfigured (missing token endpoint)")
            .request_async(async_http_client)
            .await;

        Ok(token_response?)
    }

    pub fn id_token_verifier(&'_ self) -> CoreIdTokenVerifier<'_> {
        self.client.id_token_verifier()
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.as_ref().clone()
    }
}

impl FromRef<AppState> for Arc<Mutex<HashMap<String, SessionData>>> {
    fn from_ref(state: &AppState) -> Self {
        state.sessions.clone()
    }
}

impl FromRef<AppState> for ISPOidcClient {
    fn from_ref(state: &AppState) -> Self {
        state.http_client.as_ref().clone()
    }
}

impl FromRef<AppState> for reqwest::Client {
    fn from_ref(state: &AppState) -> Self {
        state.async_http_client.clone()
    }
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

fn apply_validated_id_token_claims(
    session: &mut SessionData,
    session_id: &str,
    validated: ValidatedIdTokenData,
) -> bool {
    debug!(
        session_id = %session_id,
        subject = %validated.subject,
        "callback: id_token claims validated"
    );
    session.subject = Some(validated.subject);
    session.id_token_expires_at = Some(Instant::now() + validated.expires_in);
    session.roles = validated.roles;
    session.name = Some(validated.name);
    session.email = validated.email;
    session.id_token = Some(validated.id_token);

    let roles_extracted = !session.roles.is_empty();
    debug!(
        session_id = %session_id,
        has_name = session.name.is_some(),
        has_email = session.email.is_some(),
        roles_count = session.roles.len(),
        "callback: claims extracted into session"
    );

    roles_extracted
}

async fn take_logout_state(
    sessions: &Arc<Mutex<HashMap<String, SessionData>>>,
    chat_sessions: &Arc<Mutex<HashMap<String, Arc<ChatSession>>>>,
    session_id: &str,
) -> (Option<SessionData>, Option<String>) {
    let session = sessions.lock().await.remove(session_id);
    let chat_session = chat_sessions.lock().await.remove(session_id);
    let request_id = match chat_session {
        Some(chat_session) => chat_session.current_request_id.read().await.clone(),
        None => None,
    };

    (session, request_id)
}

pub async fn logout_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let mut post_logout_redirect_uri = "/".to_string();
    let mut rauthy_logout_url = None;
    const LOGOUT_ID_TOKEN_MIN_TTL: Duration = Duration::from_secs(30);

    if let Some(cookie) = jar.get(SESSION_ID) {
        let session_id = cookie.value().to_string();
        let (session, request_id) =
            take_logout_state(&state.sessions, &state.chat_sessions, &session_id).await;

        if let Some(request_id) = request_id {
            let client = state.async_http_client.clone();
            let agent_api_url = state.http_client.config.chat_config.agent_api_url.clone();
            let agent_secret = state
                .http_client
                .config
                .chat_config
                .agent_api_key
                .clone()
                .unwrap_or_default();
            crate::stop::cancel_agent_request(&request_id, agent_api_url, agent_secret, client);
        }

        if let Some(session) = session {
            if let Some(id_token) = session.id_token
                && session
                    .id_token_expires_at
                    .is_some_and(|exp_at| exp_at > Instant::now() + LOGOUT_ID_TOKEN_MIN_TTL)
            {
                let issuer_url = match std::env::var("OIDC_ISSUER_URL") {
                    Ok(url) => Some(url),
                    Err(_) => {
                        post_logout_redirect_uri = "/".to_string();
                        None
                    }
                };
                if let Some(issuer_url) = issuer_url {
                    let base_logout_url =
                        format!("{}/oidc/logout", issuer_url.trim_end_matches('/'));

                    post_logout_redirect_uri = state
                        .http_client
                        .config
                        .oidc_post_logout_redirect_uri
                        .clone();
                    match url::Url::parse(&base_logout_url) {
                        Ok(mut url) => {
                            url.query_pairs_mut()
                                .append_pair("id_token_hint", &id_token)
                                .append_pair("post_logout_redirect_uri", &post_logout_redirect_uri);

                            rauthy_logout_url = Some(url.to_string());
                        }
                        Err(error) => {
                            warn!(error = ?error, "logout: invalid OIDC logout URL");
                        }
                    }
                }
            }
        }
    }

    let cookie_config = &state.http_client.config.cookie_config;
    let jar = jar.remove(
        Cookie::build(SESSION_ID)
            .path(cookie_config.path.clone())
            .http_only(cookie_config.http_only)
            .secure(cookie_config.secure)
            .same_site(cookie_config.same_site.clone().into()),
    );

    match rauthy_logout_url {
        Some(url) => (jar, Redirect::to(&url)).into_response(),
        None => (jar, Redirect::to(&post_logout_redirect_uri)).into_response(),
    }
}
pub async fn leptos_server_fn_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    req: axum::extract::Request<axum::body::Body>,
) -> Response<axum::body::Body> {
    let headers = req.headers().clone();
    let auth_state = get_auth_state(state.clone(), headers).await;
    handle_server_fns_with_context(
        move || {
            let client_config = crate::ClientConfig {
                media_proxy: state.http_client.config.media_proxy.clone(),
            };
            provide_context(client_config);
            provide_context(state.sessions.clone());
            provide_context(jar.clone());
            provide_context(auth_state.clone());
        },
        req,
    )
    .await
    .into_response()
}

pub async fn leptos_main_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    req: axum::http::Request<axum::body::Body>,
) -> Response {
    let headers = req.headers().clone();
    let auth_state = get_auth_state(state.clone(), headers).await;
    let leptos_options = state.leptos_options.as_ref().clone();
    let nonce = req
        .extensions()
        .get::<leptos::nonce::Nonce>()
        .cloned()
        .unwrap_or_else(leptos::nonce::Nonce::new);
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            let client_config = crate::ClientConfig {
                media_proxy: state.http_client.config.media_proxy.clone(),
            };
            provide_context(client_config);
            provide_context(jar.clone());
            provide_context(state.sessions.clone());
            provide_context(auth_state.clone());
            provide_context(nonce.clone());
        },
        //move || view! { <App/> },
        move || shell(leptos_options.clone()),
    );
    handler(req).await.into_response()
}

pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Query(query): Query<LoginQuery>,
) -> impl IntoResponse {
    let request_id = correlation_id(&headers);
    let language = auth_language(&jar, query.lang.as_deref());
    let started_at = Instant::now();
    let discovery_url = oidc_discovery_url(&state.http_client.config.oidc_issuer_url);
    let sso_status = state
        .sso_http_client
        .get(discovery_url)
        .timeout(SSO_PREFLIGHT_TIMEOUT)
        .send()
        .await;

    match sso_status {
        Ok(response) if response.status().is_success() => {
            info!(
                request_id = %request_id,
                elapsed_ms = started_at.elapsed().as_millis(),
                "login: SSO preflight ok"
            );
        }
        Ok(response) => {
            warn!(
                request_id = %request_id,
                status = %response.status(),
                elapsed_ms = started_at.elapsed().as_millis(),
                "login: SSO preflight returned an error"
            );
            return sso_unavailable_response(
                language,
                &request_id,
                StatusCode::SERVICE_UNAVAILABLE,
            );
        }
        Err(error) => {
            warn!(
                request_id = %request_id,
                error = ?error,
                elapsed_ms = started_at.elapsed().as_millis(),
                "login: SSO preflight failed"
            );
            return sso_unavailable_response(
                language,
                &request_id,
                StatusCode::SERVICE_UNAVAILABLE,
            );
        }
    }

    let (auth_url, csrf_token, nonce, pkce_verifier) = state.http_client.authorize_url();

    let session_id = Uuid::now_v7().to_string();
    debug!(request_id = %request_id, session_id = %session_id, "login: creating pre-auth session");

    state.sessions.lock().await.insert(
        session_id.clone(),
        SessionData {
            auth_flow_id: request_id,
            csrf_token,
            nonce,
            pkce_verifier: Arc::new(Mutex::new(Some(pkce_verifier))),
            id_token: None,
            refresh_token: None,
            subject: None,
            name: None,
            roles: HashSet::new(),
            id_token_expires_at: None,
            is_refreshing: Mutex::new(false).into(),
            email: None,
        },
    );

    let cookie_config = &state.http_client.config.cookie_config;
    let jar = jar.add(
        Cookie::build((SESSION_ID, session_id))
            .path(cookie_config.path.clone())
            .http_only(cookie_config.http_only)
            .secure(cookie_config.secure)
            .same_site(cookie_config.same_site.clone().into())
            .max_age(CookieDuration::seconds(cookie_config.max_age_secs)),
    );

    (jar, Redirect::to(auth_url.as_str())).into_response()
}

pub async fn readiness_handler(State(state): State<AppState>) -> Response {
    let request_id = Uuid::now_v7();
    let started_at = Instant::now();
    let discovery_url = oidc_discovery_url(&state.http_client.config.oidc_issuer_url);
    let response = state.sso_http_client.get(discovery_url).send().await;

    let sso_available = match response {
        Ok(response) if response.status().is_success() => true,
        Ok(response) => {
            warn!(
                request_id = %request_id,
                status = %response.status(),
                elapsed_ms = started_at.elapsed().as_millis(),
                "readiness: SSO discovery returned an error"
            );
            false
        }
        Err(error) => {
            warn!(
                request_id = %request_id,
                error = ?error,
                elapsed_ms = started_at.elapsed().as_millis(),
                "readiness: SSO discovery failed"
            );
            false
        }
    };
    let (status, body) =
        readiness_status(sso_available, request_id, started_at.elapsed().as_millis());

    (status, Json(body)).into_response()
}
/// For both: leptos_main_handler and leptos_server_fn_handler
async fn get_auth_state(state: AppState, headers: HeaderMap) -> Auth {
    // extract Session ID from cookie
    let session_id = headers
        .get(http::header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|cookie_str| {
                if let Ok(cookie) = Cookie::parse(cookie_str.trim())
                    && cookie.name() == SESSION_ID
                {
                    return Some(cookie.value().to_owned());
                }
                None
            })
        });

    if let Some(id) = session_id {
        debug!(session_id = %id, "get_auth_state: session cookie present");
        if let Some(data) = get_and_refresh_session(&state, &id).await {
            debug!(
                session_id = %id,
                has_subject = data.subject.is_some(),
                has_name = data.name.is_some(),
                has_id_token = data.id_token.is_some(),
                has_refresh_token = data.refresh_token.is_some(),
                roles_count = data.roles.len(),
                expires_at_present = data.id_token_expires_at.is_some(),
                "get_auth_state: session data found"
            );
            Auth::try_from(&data).unwrap_or(Auth::Unauthenticated)
        } else {
            debug!(session_id = %id, "get_auth_state: session not found or expired");
            Auth::Unauthenticated
        }
    } else {
        debug!("get_auth_state: no session cookie");
        Auth::Unauthenticated
    }
}

pub async fn callback_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    uri: OriginalUri,
) -> impl IntoResponse {
    let request_id = correlation_id(&headers);
    let language = auth_language(&jar, None);
    let callback_started_at = Instant::now();
    let query_string = match uri.query() {
        Some(s) => s.to_string(),
        None => {
            warn!(request_id = %request_id, "callback: missing query string");
            return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
        }
    };

    let query_result: Result<CallbackQuery, _> =
        tokio::task::spawn_blocking(move || serde_urlencoded::from_str(&query_string))
            .await
            .unwrap_or_else(|_| Err(UrlError::custom("Tokio error")));

    let query: CallbackQuery = match query_result {
        Ok(q) => q,
        Err(_) => {
            warn!(request_id = %request_id, "callback: invalid query parameters");
            return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
        }
    };

    let Some(session_cookie) = jar.get(SESSION_ID) else {
        warn!(request_id = %request_id, "callback: missing session cookie");
        return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
    };

    let session_id = session_cookie.value().to_string();
    debug!(request_id = %request_id, session_id = %session_id, "callback: session cookie present");

    let (csrf_matches, nonce, pkce_verifier_slot, auth_flow_id) = {
        let sessions = state.sessions.lock().await;
        let Some(session) = sessions.get(&session_id) else {
            debug!(request_id = %request_id, session_id = %session_id, "callback: session not found in memory");
            return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
        };
        (
            session.csrf_token.secret() == &query.state,
            session.nonce.clone(),
            session.pkce_verifier.clone(),
            session.auth_flow_id.clone(),
        )
    };
    let callback_request_id = request_id;
    let request_id = auth_flow_id;
    debug!(
        request_id = %request_id,
        callback_request_id = %callback_request_id,
        session_id = %session_id,
        "callback: session found in memory"
    );

    if !csrf_matches {
        warn!(request_id = %request_id, session_id = %session_id, "callback: CSRF validation failed");
        return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
    }

    let pkce_verifier = match pkce_verifier_slot.lock().await.take() {
        Some(verifier) => verifier,
        None => {
            warn!(request_id = %request_id, session_id = %session_id, "callback: missing PKCE verifier");
            return callback_invalid_response(language, &request_id, StatusCode::BAD_REQUEST);
        }
    };

    let code = AuthorizationCode::new(query.code.clone());
    let http_client = &state.sso_http_client;

    let exchange_started_at = Instant::now();
    let exchange_result = tokio::time::timeout(
        SSO_TOKEN_EXCHANGE_TIMEOUT,
        state
            .http_client
            .exchange_code(code, pkce_verifier, http_client),
    )
    .await;

    match exchange_result {
        Err(_) => {
            warn!(
                request_id = %request_id,
                session_id = %session_id,
                elapsed_ms = exchange_started_at.elapsed().as_millis(),
                "callback: token exchange timed out"
            );
            sso_unavailable_response(language, &request_id, StatusCode::GATEWAY_TIMEOUT)
        }
        Ok(Err(error)) => {
            warn!(
                request_id = %request_id,
                session_id = %session_id,
                error = ?error,
                elapsed_ms = exchange_started_at.elapsed().as_millis(),
                "callback: token exchange failed"
            );
            sso_unavailable_response(language, &request_id, StatusCode::BAD_GATEWAY)
        }
        Ok(Ok(token_response)) => {
            debug!(request_id = %request_id, session_id = %session_id, "callback: token exchange ok");
            let mut validated_token = None;
            let mut validation_failed = false;

            if let Some(id_token) = token_response.extra_fields().id_token() {
                debug!(request_id = %request_id, session_id = %session_id, "callback: id_token present");
                match id_token.claims(&state.http_client.id_token_verifier(), &nonce) {
                    Ok(claims) => {
                        validated_token =
                            Some(validated_id_token_data(id_token.to_string(), claims));
                    }
                    Err(error) => {
                        warn!(
                            request_id = %request_id,
                            session_id = %session_id,
                            error = ?error,
                            "callback: id_token claims validation failed; retrying with fresh OIDC discovery"
                        );
                        match tokio::time::timeout(
                            SSO_DISCOVERY_TIMEOUT,
                            ISPOidcClient::new(&state.sso_http_client),
                        )
                        .await
                        {
                            Ok(Ok(fresh_client)) => {
                                match id_token.claims(&fresh_client.id_token_verifier(), &nonce) {
                                    Ok(claims) => {
                                        debug!(
                                            request_id = %request_id,
                                            session_id = %session_id,
                                            "callback: id_token claims validated after fresh OIDC discovery"
                                        );
                                        validated_token = Some(validated_id_token_data(
                                            id_token.to_string(),
                                            claims,
                                        ));
                                    }
                                    Err(retry_error) => {
                                        validation_failed = true;
                                        warn!(
                                            request_id = %request_id,
                                            session_id = %session_id,
                                            error = ?retry_error,
                                            "callback: id_token claims validation still failed after fresh OIDC discovery"
                                        );
                                    }
                                }
                            }
                            Ok(Err(refresh_error)) => {
                                validation_failed = true;
                                warn!(
                                    request_id = %request_id,
                                    session_id = %session_id,
                                    error = ?refresh_error,
                                    "callback: fresh OIDC discovery failed after claims validation error"
                                );
                            }
                            Err(_) => {
                                validation_failed = true;
                                warn!(
                                    request_id = %request_id,
                                    session_id = %session_id,
                                    elapsed_ms = callback_started_at.elapsed().as_millis(),
                                    "callback: fresh OIDC discovery timed out"
                                );
                            }
                        }
                    }
                }
            } else {
                debug!(request_id = %request_id, session_id = %session_id, "callback: id_token missing in token response");
            }

            if validation_failed {
                return sso_unavailable_response(language, &request_id, StatusCode::BAD_GATEWAY);
            }

            let mut sessions = state.sessions.lock().await;
            let Some(session) = sessions.get_mut(&session_id) else {
                warn!(request_id = %request_id, session_id = %session_id, "callback: session disappeared before update");
                return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
            };
            let roles_extracted = validated_token
                .map(|validated| apply_validated_id_token_claims(session, &session_id, validated))
                .unwrap_or(false);

            if !roles_extracted {
                let access_token = token_response.access_token();
                if let Some(access_token_claims) = extract_claims_from_access_token(access_token) {
                    session.roles = extract_roles_from_claims(&access_token_claims);
                    if session.email.is_none() {
                        session.email = extract_email_from_claims(&access_token_claims);
                    }
                }
            }

            session.refresh_token = token_response
                .refresh_token()
                .map(|t| t.secret().to_string());
            debug!(
                request_id = %request_id,
                session_id = %session_id,
                authenticated_ready = session.subject.is_some() && session.name.is_some(),
                has_refresh_token = session.refresh_token.is_some(),
                elapsed_ms = callback_started_at.elapsed().as_millis(),
                "callback: session update complete"
            );

            Redirect::to("/").into_response()
        }
    }
}

#[cfg(test)]
mod auth_reliability_tests {
    use super::*;
    #[test]
    fn auth_language_accepts_german_variants() {
        assert_eq!(AuthLanguage::from_hint(Some("de")), AuthLanguage::De);
        assert_eq!(AuthLanguage::from_hint(Some("de-DE")), AuthLanguage::De);
        assert_eq!(AuthLanguage::from_hint(Some("en")), AuthLanguage::En);
        assert_eq!(AuthLanguage::from_hint(None), AuthLanguage::En);
    }

    #[tokio::test]
    async fn logout_removes_chat_state_even_without_oidc_session() {
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let chat_sessions = Arc::new(Mutex::new(HashMap::from([(
            "session-1".to_string(),
            Arc::new(ChatSession {
                current_request_id: tokio::sync::RwLock::new(Some("request-1".to_string())),
            }),
        )])));

        let (session, request_id) = take_logout_state(&sessions, &chat_sessions, "session-1").await;

        assert!(session.is_none());
        assert_eq!(request_id.as_deref(), Some("request-1"));
        assert!(!chat_sessions.lock().await.contains_key("session-1"));
    }

    #[test]
    fn discovery_url_handles_trailing_slash() {
        assert_eq!(
            oidc_discovery_url("https://sso.example.test/"),
            "https://sso.example.test/.well-known/openid-configuration"
        );
    }

    #[test]
    fn unavailable_page_is_html_with_requested_status() {
        let response = sso_unavailable_response(
            AuthLanguage::De,
            &Uuid::nil().to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        );
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn invalid_callback_page_is_localized_html() {
        let response = callback_invalid_response(
            AuthLanguage::De,
            "proxy-request-123",
            StatusCode::BAD_REQUEST,
        );
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
    }

    #[test]
    fn correlation_id_accepts_safe_proxy_value_and_rejects_unsafe_input() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("caddy-123.trace"));
        assert_eq!(correlation_id(&headers), "caddy-123.trace");

        headers.insert("x-request-id", HeaderValue::from_static("unsafe/value"));
        assert!(Uuid::parse_str(&correlation_id(&headers)).is_ok());
    }

    #[test]
    fn readiness_distinguishes_ui_from_sso_failure() {
        let (status, body) = readiness_status(false, Uuid::nil(), 123);

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body.status, "degraded");
        assert_eq!(body.ui, "available");
        assert_eq!(body.sso, "unavailable");
        assert_eq!(body.request_id, Uuid::nil().to_string());
        assert_eq!(body.elapsed_ms, 123);
    }
}

/// Extract claims from Access Token, (if it is JWT).
pub fn extract_claims_from_access_token(token: &AccessToken) -> Option<Value> {
    let token_str = token.secret();
    let parts: Vec<&str> = token_str.split('.').collect();

    if parts.len() != 3 {
        warn!(
            "Access Token is not JWT (must b 3 part, found: {})",
            parts.len()
        );
        return None;
    }

    let payload_base64 = parts[1];

    match BASE64_URL_SAFE_NO_PAD.decode(payload_base64) {
        Ok(decoded_bytes) => match serde_json::from_slice(&decoded_bytes) {
            Ok(claims_value) => Some(claims_value),
            Err(e) => {
                warn!("Failed to deserialize Claims from Access Token: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Unable to decode Base64 Payload from Access Token: {}", e);
            None
        }
    }
}

/// Middleware, with CSP 3 && security headers
/// && nonce in Request Extensions, for Leptos
pub async fn security_headers(
    State(app_state): State<AppState>,
    mut req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let uri = req.uri().path().to_string();
    // ❌ We do not add CSP for static or API
    if is_static_asset(&uri) {
        let mut res = next.run(req).await;
        let headers = res.headers_mut();
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Expires", HeaderValue::from_static("0"));
        return res;
    }
    if is_static(uri) {
        return next.run(req).await;
    }
    // tracing::info!(">> security_headers called for {}", req.uri());
    let config = &app_state.http_client.config;
    let is_prod = config.is_prod;
    let trust_data_list = &config.trust_data_list.replace(",", " ");
    let trust_connect_list = &config.trust_connect_list;
    let nonce = leptos::nonce::Nonce::new(); //use_nonce().unwrap();
    req.extensions_mut().insert(nonce.clone());
    let mut res = next.run(req).await;

    // Content-Security-Policy
    let csp = if !is_prod {
        // DEV: relaxed to support HMR/hydration;
        // include nonce as we prepare to remove 'unsafe-inline' later
        format!(
            "default-src 'self'; \
                frame-ancestors 'none'; \
                script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' 'nonce-{}'; \
                style-src 'self' 'unsafe-inline' {trust_data_list} 'nonce-{}'; \
                img-src 'self' data: blob: {trust_data_list}; \
                font-src 'self' data: {trust_data_list}; \
                connect-src 'self' ws: wss: {trust_connect_list}",
            nonce, nonce
        )
    } else {
        // PROD: no ws/wss and no 'unsafe-eval'; keep 'unsafe-inline' for styles,
        // include nonce for scripts/styles
        format!(
            "default-src 'self';\
                 frame-ancestors 'none'; \
                 script-src 'self' 'nonce-{}' 'wasm-unsafe-eval'; \
                 style-src 'self' {trust_data_list} 'nonce-{}'; \
                 img-src 'self' data: blob: {trust_connect_list}; \
                 font-src 'self' data: {trust_data_list}; \
                 connect-src 'self' {trust_connect_list}",
            nonce, nonce
        )
    };
    let headers = res.headers_mut();
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_str(&csp).unwrap(),
    );
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("Referrer-Policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );

    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    // HSTS only in PROD (assumes HTTPS termination in front)
    if is_prod {
        headers.insert(
            "Strict-Transport-Security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    res
}

fn is_static_asset(uri: &str) -> bool {
    uri.starts_with("/pkg")
        || uri.starts_with("/assets")
        || uri.ends_with(".js")
        || uri.ends_with(".json")
        || uri.ends_with(".css")
        || uri.ends_with(".wasm")
        || uri.ends_with(".map")
        || uri.ends_with(".ico")
        || uri.ends_with(".tfl")
}

fn is_static(uri: String) -> bool {
    is_static_asset(&uri)
        || uri.starts_with("/api")
        || uri.starts_with("/login")
        || uri.starts_with("/logout")
        || uri.starts_with("/callback")
        || uri.starts_with("/local")
}
