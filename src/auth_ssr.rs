use crate::auth::*;
use crate::state::AppState;
use axum::{
    Error,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::serde_json;
use oauth2::{CsrfToken, PkceCodeVerifier, RefreshToken, TokenResponse};
use openidconnect::{
    Nonce,
    core::{CoreIdToken, CoreIdTokenClaims},
};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Only Server side
#[derive(Debug, Clone)]
pub struct SessionData {
    pub auth_flow_id: String,
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
    pub pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub subject: Option<String>,
    pub name: Option<String>,
    pub roles: HashSet<Role>,
    pub id_token_expires_at: Option<Instant>,
    pub refresh_lock: Arc<Mutex<()>>,
    pub email: Option<String>,
}
const REFRESH_THRESHOLD: Duration = Duration::from_secs(5 * 60);

pub(crate) async fn authenticated_operator(
    state: &AppState,
    cookies: &CookieJar,
) -> Result<String, StatusCode> {
    let cookie = cookies.get(SESSION_ID).ok_or(StatusCode::UNAUTHORIZED)?;
    let session = get_and_refresh_session(state, cookie.value())
        .await
        .filter(|session| session.id_token.is_some() && session.subject.is_some())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !session.roles.contains(&Role::User) && !session.roles.contains(&Role::Admin) {
        return Err(StatusCode::FORBIDDEN);
    }
    session
        .email
        .filter(|email| !email.is_empty())
        .ok_or(StatusCode::FORBIDDEN)
}

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
            email: claims
                .email()
                .map(|e| e.to_string())
                .or_else(|| session.email.clone()),
            roles: session.roles.clone(),
        })
    } else {
        // Failed to convert claims to JSON, fallback name
        tracing::info!("from id_token claims");
        Ok(AuthenticatedUser {
            subject: claims.subject().to_string(),
            name: claims.subject().to_string(),
            email: claims
                .email()
                .map(|e| e.to_string())
                .or_else(|| session.email.clone()),
            roles: session.roles.clone(),
        })
    }
}

fn refresh_token_after_rotation(current: String, rotated: Option<String>) -> String {
    rotated.unwrap_or(current)
}

fn expiry_instant_from(
    expiry: SystemTime,
    system_now: SystemTime,
    instant_now: Instant,
) -> Instant {
    instant_now + expiry.duration_since(system_now).unwrap_or(Duration::ZERO)
}

fn id_token_expiry_instant(claims: &CoreIdTokenClaims) -> Instant {
    expiry_instant_from(
        claims.expiration().into(),
        SystemTime::now(),
        Instant::now(),
    )
}

fn session_expired(session: &SessionData) -> bool {
    session
        .id_token_expires_at
        .is_some_and(|expiry| Instant::now() >= expiry)
}

pub async fn get_and_refresh_session(state: &AppState, session_id: &str) -> Option<SessionData> {
    let session = state.sessions.lock().await.get(session_id).cloned()?;
    if session_expired(&session) {
        session.refresh_token.as_ref()?;
        let _guard = session.refresh_lock.lock().await;
        return refresh_session(state, session_id, true).await;
    }

    let near_expiry = session.id_token_expires_at.is_some_and(|expiry| {
        expiry.saturating_duration_since(Instant::now()) <= REFRESH_THRESHOLD
    });
    if near_expiry
        && session.refresh_token.is_some()
        && let Ok(guard) = session.refresh_lock.clone().try_lock_owned()
    {
        let state = state.clone();
        let session_id = session_id.to_owned();
        tokio::spawn(async move {
            let _guard = guard;
            refresh_session(&state, &session_id, false).await;
        });
    }
    Some(session)
}

// The caller holds the session's refresh lock. Read again in case another
// request refreshed the token while this request was waiting.
async fn refresh_session(
    state: &AppState,
    session_id: &str,
    expired_only: bool,
) -> Option<SessionData> {
    let session = state.sessions.lock().await.get(session_id).cloned()?;
    if expired_only && !session_expired(&session) {
        return Some(session);
    }
    let token = session.refresh_token?;
    let refreshed = match perform_token_refresh(
        token.clone(),
        &state.http_client,
        &state.sso_http_client,
    )
    .await
    {
        Ok(tokens) => tokens,
        Err(_) => {
            debug!("session refresh: provider rejected refresh");
            return None;
        }
    };
    if session.subject.as_deref() != Some(refreshed.subject.as_str())
        || refreshed.expires_at <= Instant::now()
    {
        debug!("session refresh: invalid subject or expiry");
        return None;
    }
    let mut sessions = state.sessions.lock().await;
    let current = sessions.get_mut(session_id)?;
    current.id_token = Some(refreshed.id_token);
    current.refresh_token = Some(refresh_token_after_rotation(token, refreshed.refresh_token));
    current.id_token_expires_at = Some(refreshed.expires_at);
    current.roles = refreshed.roles;
    Some(current.clone())
}

pub fn trace_time(text: &str, id_token_expires_at: &Option<Instant>) {
    if let Some(expiry_instant) = id_token_expires_at {
        let duration_left = expiry_instant.saturating_duration_since(Instant::now());
        if let Ok(chrono_duration) = chrono::Duration::from_std(duration_left) {
            let local_expiry = chrono::Local::now() + chrono_duration;
            tracing::info!(
                "{} (Local): {}",
                text,
                local_expiry.format("%Y-%m-%d %H:%M:%S")
            );
        }
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
            let Some(local_session) = sessions.get(&session_id) else {
                return Err(Redirect::to("/login").into_response());
            };
            session = local_session.clone();
            drop(sessions);
        }

        let verifier = state.http_client.id_token_verifier();
        let http_client = &state.sso_http_client;

        if let Some(id_token_str) = &session.id_token {
            let id_token = match CoreIdToken::from_str(id_token_str) {
                Ok(t) => t,
                Err(_) => return Err(Redirect::to("/login").into_response()),
            };
            return match id_token.claims(&verifier, &session.nonce) {
                Ok(claims) => Ok(extract_auth_user(&session, claims).unwrap()),
                Err(_) => {
                    if let Some(refresh_token) = &session.refresh_token
                        && let Ok(new_tokens) = state
                            .http_client
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
                            session.id_token_expires_at = Some(id_token_expiry_instant(claims));
                            let email = claims
                                .email()
                                .map(|e| e.to_string())
                                .or_else(|| session.email.clone());

                            info!("from session.id_token");
                            return Ok(AuthenticatedUser {
                                subject: subject.clone(),
                                name: name.clone(),
                                email: email.clone(),
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
                email: data.email.clone(),
                roles: data.roles.clone(),
            };
            Ok(Auth::Authenticated(user))
        } else {
            Ok(Auth::Unauthenticated)
        }
    }
}

struct RefreshedSession {
    id_token: String,
    refresh_token: Option<String>,
    expires_at: Instant,
    subject: String,
    roles: HashSet<Role>,
}

async fn perform_token_refresh(
    current_refresh_token: String,
    oidc_client: &crate::ssr::ISPOidcClient,
    http_client: &reqwest::Client,
) -> Result<RefreshedSession, Box<dyn std::error::Error + Send + Sync>> {
    let refresh_token = RefreshToken::new(current_refresh_token);

    let token_response = oidc_client
        .exchange_refresh_token(&refresh_token, http_client)
        .await
        .map_err(|e| format!("Refresh token request failed: {:?}", e))?;

    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or("Missing ID Token after refresh")?;

    let claims = id_token
        .claims(
            &oidc_client.id_token_verifier(),
            |_nonce: Option<&openidconnect::Nonce>| -> Result<(), String> { Ok(()) },
        )
        .map_err(|e| format!("ID Token validation failed after refresh: {:?}", e))?;

    let mut roles = serde_json::to_value(claims)
        .ok()
        .map(|claims| extract_roles_from_claims(&claims))
        .unwrap_or_default();
    if roles.is_empty()
        && let Some(claims) =
            crate::ssr::extract_claims_from_access_token(token_response.access_token())
    {
        roles = extract_roles_from_claims(&claims);
    }
    Ok(RefreshedSession {
        id_token: id_token.to_string(),
        refresh_token: token_response
            .refresh_token()
            .map(|t| t.secret().to_string()),
        expires_at: id_token_expiry_instant(claims),
        subject: claims.subject().to_string(),
        roles,
    })
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
pub fn extract_email_from_claims(claims: &serde_json::Value) -> Option<String> {
    claims
        .get("email")
        .and_then(|v| v.as_str())
        .or_else(|| claims.get("preferred_username").and_then(|v| v.as_str()))
        .or_else(|| claims.get("upn").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
}

#[cfg(test)]
mod refresh_tests {
    use super::{expiry_instant_from, refresh_token_after_rotation};
    use std::time::{Duration, Instant, SystemTime};

    #[test]
    fn refresh_token_is_preserved_when_provider_does_not_rotate_it() {
        assert_eq!(
            refresh_token_after_rotation("current".to_string(), None),
            "current"
        );
        assert_eq!(
            refresh_token_after_rotation("current".to_string(), Some("rotated".to_string()),),
            "rotated"
        );
    }

    #[test]
    fn refreshed_expiry_uses_actual_token_expiration() {
        let system_now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let instant_now = Instant::now();

        assert_eq!(
            expiry_instant_from(
                system_now + Duration::from_secs(600),
                system_now,
                instant_now,
            ),
            instant_now + Duration::from_secs(600)
        );
        assert_eq!(
            expiry_instant_from(system_now - Duration::from_secs(1), system_now, instant_now,),
            instant_now
        );
    }
}
