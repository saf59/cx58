use crate::app::shell;
use crate::auth::*;
use crate::auth_ssr::*;
use crate::config::AppConfig;
use crate::state::AppState;
use axum::{
    extract::{FromRef, OriginalUri, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
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
    CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreGenderClaim, CoreIdTokenVerifier,
    CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata, CoreResponseType,
    CoreTokenIntrospectionResponse, CoreTokenResponse,
};
use openidconnect::{
    AuthenticationFlow, EmptyAdditionalClaims, IssuerUrl, Nonce, OAuth2TokenResponse,
};
use serde::{Deserialize, de::Error};
use serde_json::Value;
use serde_urlencoded::de::Error as UrlError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
#[allow(unused_imports)]
use tracing::{info, warn};
use uuid::Uuid;

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
        state.oidc_client.as_ref().clone()
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

pub async fn logout_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let mut post_logout_redirect_uri = "/".to_string();
    let mut rauthy_logout_url = None;

    if let Some(cookie) = jar.get(SESSION_ID) {
        let session_id = cookie.value().to_string();
        let mut sessions = state.sessions.lock().await;

        if let Some(session) = sessions.remove(&session_id)
            && let Some(id_token) = session.id_token
        {
            let mut chat_sessions = state.chat_sessions.lock().await;
            // Cancel active requests
            if let Some(chat_session) = chat_sessions.get_mut(&session_id) {
                let _ = chat_session.cancel_tx.send(true);
                if let Some(request_id) = &chat_session.current_request_id.read().await.clone() {
                    let client = state.async_http_client.clone();
                    let agent_api_url = state.oidc_client.config.chat_config.agent_api_url.clone();
                    crate::stop::cancel_agent_request(request_id, agent_api_url, client);
                }
            }

            let issuer_url = match std::env::var("OIDC_ISSUER_URL") {
                Ok(url) => url,
                Err(_) => {
                    post_logout_redirect_uri = "/".to_string();
                    return (jar, Redirect::to(&post_logout_redirect_uri)).into_response();
                }
            };
            let base_logout_url = format!("{}/oidc/logout", issuer_url.trim_end_matches('/'));

            post_logout_redirect_uri = state
                .oidc_client
                .config
                .oidc_post_logout_redirect_uri
                .clone();
            let mut url = url::Url::parse(&base_logout_url).expect("Invalid base logout URL");

            url.query_pairs_mut()
                .append_pair("id_token_hint", &id_token)
                .append_pair("post_logout_redirect_uri", &post_logout_redirect_uri);

            rauthy_logout_url = Some(url.to_string());
        }
    }

    let jar = jar.remove(Cookie::from(SESSION_ID));

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
            provide_context(jar.clone());
            provide_context(state.sessions.clone());
            provide_context(auth_state.clone());
            provide_context(nonce.clone());
        },
        move || shell(leptos_options.clone()),
    );
    handler(req).await.into_response()
}

pub async fn login_handler(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    let (auth_url, csrf_token, nonce, pkce_verifier) = state.oidc_client.authorize_url();

    let session_id = Uuid::now_v7().to_string();

    state.sessions.lock().await.insert(
        session_id.clone(),
        SessionData {
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

    let jar = jar.add(
        Cookie::build((SESSION_ID, session_id))
            .path("/")
            .http_only(true)
            // Recommended for OIDC flow
            .same_site(axum_extra::extract::cookie::SameSite::Lax),
    );

    (jar, Redirect::to(auth_url.as_str()))
}
/// For both: leptos_main_handler and leptos_server_fn_handler
async fn get_auth_state(state: AppState, headers: HeaderMap) -> Auth {
    // Извлечение Session ID из кук
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
        if let Some(data) = get_and_refresh_session(&state, &id).await {
            Auth::try_from(&data).unwrap_or(Auth::Unauthenticated)
        } else {
            Auth::Unauthenticated
        }
    } else {
        Auth::Unauthenticated
    }
}

pub async fn callback_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    uri: OriginalUri,
) -> impl IntoResponse {
    let query_string = match uri.query() {
        Some(s) => s.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing query string").into_response(),
    };

    let query_result: Result<CallbackQuery, _> =
        tokio::task::spawn_blocking(move || serde_urlencoded::from_str(&query_string))
            .await
            .unwrap_or_else(|_| Err(UrlError::custom("Tokio error")));

    let query: CallbackQuery = match query_result {
        Ok(q) => q,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid query parameters or task failed",
            )
                .into_response();
        }
    };

    let Some(session_cookie) = jar.get(SESSION_ID) else {
        return (StatusCode::BAD_REQUEST, "Missing session cookie").into_response();
    };

    let session_id = session_cookie.value().to_string();

    let mut sessions = state.sessions.lock().await;
    let Some(session) = sessions.get_mut(&session_id) else {
        return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
    };
    let mut pkce_guard = session.pkce_verifier.lock().await;
    let pkce_verifier_to_check = pkce_guard.take();

    if session.csrf_token.secret() != &query.state {
        return (StatusCode::BAD_REQUEST, "CSRF validation failed").into_response();
    };
    let pkce_verifier = match pkce_verifier_to_check {
        Some(verifier) => verifier,
        None => return (StatusCode::BAD_REQUEST, "Missing PKCE verifier").into_response(),
    };

    let code = AuthorizationCode::new(query.code.clone());
    let http_client = &state.async_http_client;

    match state
        .oidc_client
        .exchange_code(code, pkce_verifier, http_client)
        .await
    {
        Ok(token_response) => {
            let mut roles_extracted = false;

            if let Some(id_token) = token_response.extra_fields().id_token()
                && let Ok(claims) =
                    id_token.claims(&state.oidc_client.id_token_verifier(), &session.nonce)
            {
                session.subject = Some(claims.subject().to_string());

                let expiry_datetime_utc = claims.expiration();
                let expiry_system_time: SystemTime = expiry_datetime_utc.into();
                let duration_until_expiry = expiry_system_time
                    .duration_since(SystemTime::now())
                    .unwrap_or(Duration::ZERO); // Если время уже прошло -> 0
                session.id_token_expires_at = Some(Instant::now() + duration_until_expiry);

                if let Ok(claims_json) = serde_json::to_value(claims) {
                    session.roles = extract_roles_from_claims(&claims_json);
                    // Name is also extracted here
                    session.name = Some(extract_name_from_claims(&claims_json));
                    session.email = extract_email_from_claims(&claims_json);
                    roles_extracted = !session.roles.is_empty();
                }
                session.id_token = Some(id_token.to_string());
            }

            // Fallback for roles and name if not extracted from ID token
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

            Redirect::to("/").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Token exchange failed: {:?}", e),
        )
            .into_response(),
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
    if is_static(uri) {
        return next.run(req).await;
    }
    // tracing::info!(">> security_headers called for {}", req.uri());
    let config = &app_state.oidc_client.config;
    let is_prod = config.is_prod;
    let trust_data_list = &config.trust_data_list;
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
                img-src 'self' data: blob:; \
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
                 img-src 'self' data: blob:; \
                 font-src 'self' data: {trust_data_list}; \
                 connect-src 'self' {trust_connect_list}",
            nonce, nonce
        )
    };
    //println!("{csp}");
    let headers = res.headers_mut();
    //println!("{:?}",&headers);
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
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "Cross-Origin-Resource-Policy",
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

fn is_static(uri: String) -> bool {
    uri.starts_with("/pkg")
        || uri.starts_with("/assets")
        || uri.starts_with("/api")
        || uri.starts_with("/login")
        || uri.starts_with("/logout")
        || uri.starts_with("/callback")
        || uri.starts_with("/local")
        || uri.ends_with(".js")
        || uri.ends_with(".json")
        || uri.ends_with(".css")
        || uri.ends_with(".wasm")
        || uri.ends_with(".map")
        || uri.ends_with(".ico")
        || uri.ends_with(".tfl")
}
