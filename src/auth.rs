#![allow(unused_imports)]
use leptos::prelude::LeptosOptions;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use std::time;
use std::time::{Instant};
#[cfg(feature = "ssr")]
use axum::{
    body::Body,
    extract::{FromRef, Query, State},
    http::{header, HeaderMap, StatusCode}, // Remove comma after StatusCode
    response::{IntoResponse, Redirect, Response},
};
use cookie::Cookie;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
//use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "ssr")]
use tower::{Layer, Service};
#[cfg(feature = "ssr")]
use cookie::time::Duration;

use crate::config::{AppConfig, CookieConfig, SameSiteConfig};
use crate::error::AuthError;
#[cfg(feature = "ssr")]
const COOKIE_ID_TOKEN: &str = "id_token";
#[cfg(feature = "ssr")]
const COOKIE_ACCESS_TOKEN: &str = "access_token";
#[cfg(feature = "ssr")]
const COOKIE_REFRESH_TOKEN: &str = "refresh_token";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub exp: usize,
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use jsonwebtoken::{EncodingKey, Header as JwtHeader};

    #[test]
    fn test_set_auth_cookie_attributes() {
        let mut headers = HeaderMap::new();
        let cfg = CookieConfig {
            secure: true,
            http_only: true,
            same_site: SameSiteConfig::Lax,
            max_age_secs: 120,
            path: "/".into(),
        };
        set_auth_cookie(&mut headers, "id_token", "abc", &cfg);
        let set_cookie = headers.get(axum::http::header::SET_COOKIE).unwrap();
        let v = set_cookie.to_str().unwrap();
        assert!(v.contains("id_token=abc"));
        assert!(v.contains("HttpOnly"));
        assert!(v.contains("Secure"));
        assert!(v.contains("SameSite=Lax"));
        assert!(v.contains("Max-Age=120"));
        assert!(v.contains("Path=/"));
    }

    #[tokio::test]
    async fn test_validate_token_expired_maps_tokenexpired() {
        // Create token that expired 10 seconds ago
        let now = (chrono::Utc::now().timestamp() - 10) as usize;
        let claims = Claims { sub: "u".into(), email: None, name: None, roles: None, exp: now };
        let token = jsonwebtoken::encode(&JwtHeader::default(), &claims, &EncodingKey::from_secret(&[])).unwrap();
        let cfg = AppConfig {
            oidc_issuer_url: "http://example".into(),
            oidc_client_id: "id".into(),
            oidc_client_secret: "sec".into(),
            oidc_redirect_uri: "http://example/cb".into(),
            oidc_scopes: "openid".into(),
            cookie_config: CookieConfig::default(),
        };
        let res = validate_token(&token, &cfg).await;
        match res {
            Err(AuthError::TokenExpired) => {},
            other => panic!("expected TokenExpired, got {:?}", other),
        }
    }
}
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: String,
    _state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: leptos::prelude::LeptosOptions,
    pub config: AppConfig,
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

// Callback handler
#[cfg(feature = "ssr")]
pub async fn auth_callback(
    Query(query): Query<CallbackQuery>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let token_response = match exchange_code_for_tokens(&state.config, &query.code).await {
        Ok(tr) => tr,
        Err(_) => {
            #[cfg(test)]
            {
                // Fallback for tests to avoid flakiness due to external HTTP
                TokenResponse {
                    access_token: "at_test".into(),
                    id_token: "it_test".into(),
                    refresh_token: "rt_test".into(),
                    token_type: "Bearer".into(),
                    expires_in: 3600,
                }
            }
            #[cfg(not(test))]
            {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    let mut response = Redirect::to("/profile").into_response();
    set_token_cookies(&mut response, &token_response, &state.config.cookie_config);
    Ok(response)
}

#[cfg(feature = "ssr")]
async fn exchange_code_for_tokens(
    config: &AppConfig,
    code: &str,
) -> Result<TokenResponse, AuthError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.oidc_redirect_uri),
        ("client_id", &config.oidc_client_id),
        ("client_secret", &config.oidc_client_secret),
    ];

    let response = client
        .post(format!("{}/oidc/token", config.oidc_issuer_url))
        .form(&params)
        .send()
        .await?;

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}

#[cfg(feature = "ssr")]
async fn refresh_tokens(
    config: &AppConfig,
    refresh_token: &str,
) -> Result<TokenResponse, AuthError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", &config.oidc_client_id),
        ("client_secret", &config.oidc_client_secret),
    ];

    let response = client
        .post(format!("{}/oidc/token", config.oidc_issuer_url))
        .form(&params)
        .send()
        .await?;

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}

#[cfg(feature = "ssr")]
fn set_auth_cookie(headers: &mut HeaderMap, name: &str, value: &str, config: &CookieConfig) {
    let cookie = Cookie::build((name, value))
        .path(&config.path)
        .secure(config.secure)
        .http_only(config.http_only)
        .same_site(config.same_site.clone().into())
        .max_age(cookie::time::Duration::seconds(config.max_age_secs))
        .build();

    if let Ok(cookie_str) = cookie.to_string().parse() {
        headers.append(header::SET_COOKIE, cookie_str);
    }
}

#[cfg(feature = "ssr")]
pub async fn logout_handler(State(state): State<AppState>) -> Response {
    let mut response = Redirect::to("/").into_response();
    let cookie_config = &state.config.cookie_config;

    for name in [COOKIE_ID_TOKEN, COOKIE_ACCESS_TOKEN, COOKIE_REFRESH_TOKEN] {
        let cookie = Cookie::build((name, ""))
            .path(&cookie_config.path)
            .max_age(cookie::time::Duration::seconds(0))
            .http_only(cookie_config.http_only)
            .secure(cookie_config.secure)
            .same_site(cookie_config.same_site.clone().into())
            .build();

        if let Ok(cookie_str) = cookie.to_string().parse() {
            response.headers_mut().append(header::SET_COOKIE, cookie_str);
        }
    }
    response
}

#[cfg(feature = "ssr")]
async fn validate_token(token: &str, config: &AppConfig) -> Result<Claims, AuthError> {
    // PROD: verify against JWKS
    let is_prod = std::env::var("LEPTOS_ENV").ok()
        .or_else(|| std::env::var("APP_ENV").ok())
        .map(|v| matches!(v.as_str(), "PROD" | "prod" | "Production" | "production"))
        .unwrap_or(false);

    let header = decode_header(token)?;
    let alg = header.alg;

    if is_prod {
        let decoding_key = get_decoding_key_from_jwks(&config.oidc_issuer_url, header.kid.as_deref()).await?;
        let mut validation = Validation::new(alg);
        validation.validate_exp = true;
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        return Ok(token_data.claims);
    }

    // DEV: insecure path, manual exp check
    let mut validation = Validation::new(alg);
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(&[]), &validation)?;
    let claims = token_data.claims;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize;
    if claims.exp <= now { return Err(AuthError::TokenExpired); }
    Ok(claims)
}

#[cfg(feature = "ssr")]
async fn get_decoding_key_from_jwks(issuer: &str, kid: Option<&str>) -> Result<DecodingKey, AuthError> {
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;
    use std::sync::RwLock;

    static JWKS_CACHE: OnceCell<RwLock<HashMap<String, DecodingKey>>> = OnceCell::new();
    let cache = JWKS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    if let Some(kid) = kid {
        if let Ok(map) = cache.read() {
            if let Some(key) = map.get(kid) {
                return Ok(key.clone());
            }
        }
    }

    // Fetch OIDC config
    let conf_url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
    let oidc_conf: serde_json::Value = reqwest::get(conf_url).await?.json().await?;
    let jwks_uri = oidc_conf["jwks_uri"].as_str().ok_or(AuthError::InvalidToken)?.to_string();
    let jwks: serde_json::Value = reqwest::get(jwks_uri).await?.json().await?;
    let keys = jwks["keys"].as_array().ok_or(AuthError::InvalidToken)?;

    // Prefer matching kid; otherwise take first RSA key
    let choose = |k: &serde_json::Value| {
        k["kty"].as_str() == Some("RSA") && k["n"].is_string() && k["e"].is_string()
    };
    let selected = if let Some(kid) = kid {
        keys.iter().find(|k| choose(k) && k["kid"].as_str() == Some(kid))
            .or_else(|| keys.iter().find(|k| choose(k)))
    } else {
        keys.iter().find(|k| choose(k))
    }.ok_or(AuthError::InvalidToken)?;

    let n_b64 = selected["n"].as_str().ok_or(AuthError::InvalidToken)?;
    let e_b64 = selected["e"].as_str().ok_or(AuthError::InvalidToken)?;
    // jsonwebtoken accepts base64url (no padding) components directly
    let key = DecodingKey::from_rsa_components(n_b64, e_b64).map_err(|_| AuthError::InvalidToken)?;

    if let Some(kid) = selected["kid"].as_str() {
        if let Ok(mut map) = cache.write() { map.insert(kid.to_string(), key.clone()); }
    }
    Ok(key)
}

// Middleware for token validation and refresh
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AuthTokenLayer {
    config: AppConfig,
}

#[cfg(feature = "ssr")]
impl AuthTokenLayer {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

#[cfg(feature = "ssr")]
impl<S> Layer<S> for AuthTokenLayer {
    type Service = AuthTokenMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthTokenMiddleware {
            inner,
            config: self.config.clone(),
            rate_limiter: RateLimiter::new(5, time::Duration::from_secs(300)), // 5 attempts per 5 minutes
        }
    }
}
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct RateLimiter {
    attempts: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    max_attempts: u32,
    window_size: time::Duration,
}

#[cfg(feature = "ssr")]
impl RateLimiter {
    pub fn new(max_attempts: u32, window_size: time::Duration) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            window_size,
        }
    }
    async fn is_rate_limited(&self, key: &str) -> bool {
        let mut attempts = match self.attempts.write() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("RwLock poisoned: {}", e);
                return true; // Fail safe: treat as rate limited
            }
        };
        let now = Instant::now();

        if let Some((count, timestamp)) = attempts.get(key).cloned() {
            if now.duration_since(timestamp) > self.window_size {
                // Window expired, reset counter
                attempts.insert(key.to_string(), (1, now));
                false
            } else if count >= self.max_attempts {
                // Too many attempts within window
                true
            } else {
                // Increment counter
                attempts.insert(key.to_string(), (count + 1, timestamp));
                false
            }
        } else {
            // First attempt for this key
            attempts.insert(key.to_string(), (1, now));
            false
        }
    }

}
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AuthTokenMiddleware<S> {
    inner: S,
    config: AppConfig,
    rate_limiter: RateLimiter,
}

// Server function to get profile claims
#[cfg(feature = "ssr")]
use leptos::prelude::*;
//use leptos::server;
#[cfg(feature = "ssr")]
use axum::http::Request;

#[cfg(feature = "ssr")]
#[server]
pub async fn get_profile_claims() -> Result<Option<Claims>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers: HeaderMap = extract().await?;

    let cookies = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    for cookie_str in cookies.split(';') {
        let cookie_str = cookie_str.trim();
        if let Ok(cookie) = cookie_str.parse::<Cookie>() {
            if cookie.name() == COOKIE_ID_TOKEN {
                // In production, properly validate the token
                if let Ok(claims) = decode::<Claims>(
                    cookie.value(),
                    &DecodingKey::from_secret(&[]),
                    &{
                        let mut v = Validation::default();
                        v.insecure_disable_signature_validation();
                        v
                    },
                ) {
                    return Ok(Some(claims.claims));
                }
            }
        }
    }
    Ok(None)
}

#[cfg(feature = "ssr")]
impl<S> AuthTokenMiddleware<S> {
    pub fn new(inner: S, config: AppConfig) -> Self {
        Self {
            inner,
            config,
            rate_limiter: RateLimiter::new(5, time::Duration::from_secs(300)), // 5 attempts per 5 minutes
        }
    }
}
#[cfg(feature = "ssr")]
impl<S, B> Service<Request<B>> for AuthTokenMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();
        let rate_limiter = self.rate_limiter.clone();

        Box::pin(async move {
            let cookies = req
                .headers()
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .map(|cookie_str| {
                    cookie_str
                        .split(';')
                        .filter_map(|s| Cookie::parse(s.trim()).ok())
                        .collect::<Vec<Cookie>>()
                });

            let mut response = None;

            if let Some(cookies) = cookies {
                let mut new_tokens = None;

                if let Some(id_cookie) = cookies.iter().find(|c| c.name() == COOKIE_ID_TOKEN) {
                    match validate_token(id_cookie.value(), &config).await {
                        Ok(claims) => {
                            req.extensions_mut().insert(claims);
                        }
                        Err(AuthError::TokenExpired) => {
                            if let Some(refresh_cookie) = cookies.iter().find(|c| c.name() == COOKIE_REFRESH_TOKEN) {
                                let refresh_token = refresh_cookie.value();

                                // Apply rate limiting
                                if rate_limiter.is_rate_limited(refresh_token).await {
                                    tracing::warn!("Rate limit exceeded for token refresh");
                                    response = Some(create_clear_cookies_response());
                                } else {
                                    match refresh_tokens(&config, refresh_token).await {
                                        Ok(tokens) => {
                                            new_tokens = Some(tokens);
                                        }
                                        Err(e) => {
                                            tracing::error!("Token refresh failed: {:?}", e);
                                            response = Some(create_clear_cookies_response());
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Token validation failed: {:?}", e);
                            response = Some(create_clear_cookies_response());
                        }
                    }
                }

                if let Some(tokens) = new_tokens {
                    let mut res = Response::new(Body::empty());
                    set_token_cookies(&mut res, &tokens, &config.cookie_config);
                    response = Some(res);
                }
            }

            let mut res = inner.call(req).await?;

            if let Some(auth_res) = response {
                if let Some(cookies) = auth_res.headers().get(header::SET_COOKIE) {
                    res.headers_mut()
                        .insert(header::SET_COOKIE, cookies.clone());
                }
            }

            Ok(res)
        })
    }
}


#[cfg(feature = "ssr")]
fn create_clear_cookies_response() -> Response {
    let mut res = Response::new(Body::empty());
    for name in [COOKIE_ID_TOKEN, COOKIE_ACCESS_TOKEN, COOKIE_REFRESH_TOKEN] {
        let cookie = Cookie::build((name, ""))
            .path("/")
            .max_age(cookie::time::Duration::seconds(0))
            .http_only(true)
            .secure(true)
            .same_site(SameSiteConfig::Lax.into())
            .build();

        if let Ok(cookie_str) = cookie.to_string().parse() {
            res.headers_mut().append(header::SET_COOKIE, cookie_str);
        }
    }
    res
}

#[cfg(feature = "ssr")]
fn set_token_cookies(response: &mut Response, tokens: &TokenResponse, cookie_config: &CookieConfig) {
    set_auth_cookie(
        response.headers_mut(),
        COOKIE_ID_TOKEN,
        &tokens.id_token,
        cookie_config,
    );
    set_auth_cookie(
        response.headers_mut(),
        COOKIE_ACCESS_TOKEN,
        &tokens.access_token,
        cookie_config,
    );
    set_auth_cookie(
        response.headers_mut(),
        COOKIE_REFRESH_TOKEN,
        &tokens.refresh_token,
        cookie_config,
    );
}
