use crate::auth::*;
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
    Error,
};
use axum_extra::extract::CookieJar;
use leptos::serde_json;
use oauth2::{CsrfToken, PkceCodeVerifier, RefreshToken, TokenResponse};
use openidconnect::{core::{CoreIdToken, CoreIdTokenClaims}, Nonce};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use tracing::info;

/// Only Server side
#[derive(Debug, Clone)]
pub struct SessionData {
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
    pub pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub subject: Option<String>,
    pub name: Option<String>,
    pub roles: HashSet<Role>,
    pub id_token_expires_at: Option<Instant>,
    pub is_refreshing: Arc<Mutex<bool>>,
}
const REFRESH_THRESHOLD: Duration = Duration::from_secs(5 * 60);

pub struct SessionDataWithRefresh(pub SessionData);

fn extract_auth_user(
    session: &SessionData,
    claims: &CoreIdTokenClaims,
) -> Result<AuthenticatedUser, Error> {
    if let Ok(claims_json) = serde_json::to_value::<&_>(claims) {
        tracing::info!("from id_token claims_json");
        Ok(AuthenticatedUser {
            subject: claims.subject().to_string(),
            name: extract_name_from_claims(&claims_json), // UPDATED
            roles: session.roles.clone(),
        })
    } else {
        // Failed to convert claims to JSON, fallback name
        tracing::info!("from id_token claims");
        Ok(AuthenticatedUser {
            subject: claims.subject().to_string(),
            name: claims.subject().to_string(),
            roles: session.roles.clone(),
        })
    }
}

async fn save_session_data_in_store(
    store: &Arc<Mutex<HashMap<String, SessionData>>>,
    session_id: String,
    data: SessionData,
) {
    let mut map_guard = store.lock().await;
    map_guard.insert(session_id, data);
}

pub async fn get_and_refresh_session(
    state: &AppState,
    session_id: &str,
) -> Option<SessionData> {
    let session_data = {
        let sessions = state.sessions.lock().await;
        sessions.get(session_id).cloned()?
    };

    let now = Instant::now();

    if session_data.id_token_expires_at.is_some_and(|exp_at| now >= exp_at) {
        return None;
    }

    let needs_refresh = session_data.id_token_expires_at.is_some_and(|exp_at| {
        exp_at.checked_sub(REFRESH_THRESHOLD).is_some_and(|future_time| {
            //trace_time("Refresh time at",&Some(future_time));
            future_time <= now
        })
    });

    if needs_refresh && session_data.refresh_token.is_some() {
        let mut is_refreshing_lock = session_data.is_refreshing.lock().await;

        if !*is_refreshing_lock {
            *is_refreshing_lock = true;
            drop(is_refreshing_lock);
            let refresh_token = session_data.refresh_token.clone().unwrap();
            let session_id_clone = session_id.to_owned();
            let session_store_clone = state.sessions.clone();
            let oidc_client_clone = state.oidc_client.clone();
            let http_client_clone = state.async_http_client.clone();

            tokio::spawn(async move {
                tracing::info!("Background refresh started for session: {}", session_id_clone);

                match perform_token_refresh(
                    refresh_token,
                    &oidc_client_clone,
                    &http_client_clone
                ).await {
                    Ok((new_id_token, new_refresh_token, new_expires_at)) => {
                        let updated_session = {
                            let mut guard = session_store_clone.lock().await;
                            if let Some(current_data) = guard.get_mut(&session_id_clone) {
                                current_data.id_token = Some(new_id_token);
                                current_data.refresh_token = new_refresh_token;
                                current_data.id_token_expires_at = Some(new_expires_at);
                                *current_data.is_refreshing.lock().await = false;

                                //trace_time("Updated session ID Token expires at",&current_data.id_token_expires_at);

                                Some(current_data.clone())
                            } else {
                                None
                            }
                        };

                        if let Some(data) = updated_session {
                            save_session_data_in_store(&session_store_clone, session_id_clone, data).await;
                        }
                    },
                    Err(_e) => {
                        //tracing::error!("Token refresh failed for session {}: {:?}", session_id_clone, e);
                        let guard = session_store_clone.lock().await;
                        if let Some(current_data) = guard.get(&session_id_clone) {
                            *current_data.is_refreshing.lock().await = false;
                        }
                    }
                }
            });
        }
    }
    Some(session_data)
}

pub fn trace_time(text:&str,id_token_expires_at: &Option<Instant>) {
    if let Some(expiry_instant) = id_token_expires_at {
        let duration_left = expiry_instant.saturating_duration_since(Instant::now());
        if let Ok(chrono_duration) = chrono::Duration::from_std(duration_left) {
            let local_expiry = chrono::Local::now() + chrono_duration;
            tracing::info!("{} (Local): {}",text,local_expiry.format("%Y-%m-%d %H:%M:%S")                       ); }
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser
where
    Self: 'static,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(SESSION_ID) else {
            let response: Response = Redirect::to("/login").into_response();
            return Err(response);
        };

        let session_id = cookie.value().to_string();
        let session: SessionData;
        // 1st
        {
            let sessions = state.sessions.lock().await;
            let Some(local_session) = sessions.get(&session_id)
            else {
                return Err(Redirect::to("/login").into_response());
            };
            session = local_session.clone();
            drop(sessions);
        }

        let verifier = state.oidc_client.id_token_verifier();
        let http_client = &state.async_http_client;

        if let Some(id_token_str) = &session.id_token {
            let id_token = match CoreIdToken::from_str(id_token_str) {
                Ok(t) => t,
                Err(_) => return Err(Redirect::to("/login").into_response()),
            };

            return match id_token.claims(&verifier, &session.nonce) {
                Ok(claims) => {
                    Ok(extract_auth_user(&session, claims).unwrap())
                }
                Err(_) => {
                    if let Some(refresh_token) = &session.refresh_token
                        && let Ok(new_tokens) = state.oidc_client
                        .exchange_refresh_token(
                            &RefreshToken::new(refresh_token.clone()),
                            http_client,
                        )
                        .await
                    {
                        let mut sessions = state.sessions.lock().await;
                        let Some(session) = sessions.get_mut(&session_id) else {
                            return Err(Redirect::to("/login").into_response());
                        };

                        session.id_token =
                            new_tokens.extra_fields().id_token().map(|t| t.to_string());
                        if let Some(new_rt) = new_tokens.refresh_token() {
                            session.refresh_token = Some(new_rt.secret().to_string());
                        }

                        if let Some(idt) = &session.id_token
                            && let Ok(idt_obj) = CoreIdToken::from_str(idt)
                            && let Ok(claims) = idt_obj.claims(&verifier, &session.nonce)
                            && let Ok(claims_json) = serde_json::to_value(claims)
                        {
                            // Re-extract roles and name from refreshed token
                            let roles = extract_roles_from_claims(&claims_json);
                            let subject = claims.subject().to_string();
                            let name = extract_name_from_claims(&claims_json);
                            session.roles = roles.clone();
                            session.subject = Some(subject.clone());
                            session.name = Some(name.clone());
                            info!("from session.id_token");
                            return Ok(AuthenticatedUser {
                                subject: subject.clone(),
                                name: name.clone(),
                                roles: roles.clone(),
                            });
                        }
                    }

                    Err(Redirect::to("/login").into_response())
                }
            };
        }

        Err(Redirect::to("/login").into_response())
    }
}

impl TryFrom<&SessionData> for Auth {
    type Error = &'static str;

    fn try_from(data: &SessionData) -> Result<Self, Self::Error> {
        if let Some(name) = &data.name
            && let Some(subject) = &data.subject
        {
            let user = AuthenticatedUser {
                subject: subject.clone(),
                name: name.clone(),
                roles: data.roles.clone(),
            };
            Ok(Auth::Authenticated(user))
        } else {
            Ok(Auth::Unauthenticated)
        }
    }
}

type RefreshResult =
    Result<(String, Option<String>, Instant), Box<dyn std::error::Error + Send + Sync>>;

pub async fn perform_token_refresh(
    current_refresh_token: String,
    oidc_client: &crate::ssr::ISPOidcClient,
    http_client: &reqwest::Client,
) -> RefreshResult {
    let refresh_token = RefreshToken::new(current_refresh_token);

    let token_response = oidc_client
        .exchange_refresh_token(&refresh_token,http_client)
        .await
        .map_err(|e| format!("Refresh token request failed: {:?}", e))?;

    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or("Missing ID Token after refresh")?;

    let claims = id_token
        .claims(&oidc_client.id_token_verifier(), |_nonce: Option<&openidconnect::Nonce>| -> Result<(), String> {
            Ok(())
        },)
        .map_err(|e| format!("ID Token validation failed after refresh: {:?}", e))?;

    let exp_timestamp = claims.expiration().timestamp() as u64;

    let expires_at_system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(exp_timestamp);

    let time_until_expiry = expires_at_system_time
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);

    let duration_to_wait = time_until_expiry.saturating_sub(REFRESH_THRESHOLD);
    if duration_to_wait.is_zero() {
        tracing::info!("Refresh window reached, refreshing immediately...");
    } else {
        tracing::info!("Waiting {:?} before refreshing...", duration_to_wait);
    }
    let new_expires_at = Instant::now() + duration_to_wait;

    Ok((
        id_token.to_string(),
        token_response
            .refresh_token()
            .map(|t| t.secret().to_string()),
        new_expires_at,
    ))
}
pub fn extract_roles_from_claims(claims: &serde_json::Value) -> HashSet<Role> {
    let mut roles = HashSet::new();

    // Try different common claim names for roles
    let role_claims = [
        "roles",
        "role",
        "groups",
        "group",
        "realm_access.roles",
        "resource_access.roles",
    ];

    for claim_name in &role_claims {
        if let Some(role_value) = claims.get(claim_name) {
            match role_value {
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        if let Some(role_str) = item.as_str() {
                            roles.insert(Role::from_string(role_str));
                        }
                    }
                }
                serde_json::Value::String(s) => {
                    roles.insert(Role::from_string(s));
                }
                _ => {}
            }
        }
    }

    if let Some(roles_array) = claims.get("roles")
        && let Some(arr) = roles_array.as_array()
    {
        for item in arr {
            if let Some(role_str) = item.as_str() {
                roles.insert(Role::from_string(role_str));
            }
        }
    }

    roles
}
pub fn extract_name_from_claims(claims: &serde_json::Value) -> String {
    let first_name = claims
        .get("given_name")
        .or_else(|| claims.get("first_name"))
        .and_then(|v| v.as_str());

    let last_name = claims
        .get("family_name")
        .or_else(|| claims.get("last_name"))
        .and_then(|v| v.as_str());

    let email = claims.get("email").and_then(|v| v.as_str());

    match (first_name, last_name) {
        (Some(first), Some(last)) => format!("{} {}", first, last),
        (Some(first), None) => first.to_string(),
        (None, Some(last)) => last.to_string(),
        (None, None) => email.unwrap_or("User").to_string(), // Fallback to email or "User"
    }
}
