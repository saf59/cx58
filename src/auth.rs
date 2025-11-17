#[cfg(feature = "ssr")]
use axum::{
    Error,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter, Pointer};
use crate::rback::Role;
#[cfg(feature = "ssr")]
use crate::state::AppState;
#[cfg(feature = "ssr")]
use crate::state::SessionData;
#[cfg(feature = "ssr")]
use axum_extra::extract::cookie::Cookie;
use leptos::prelude::{RenderHtml, StorageAccess};
use leptos::serde_json;
use openidconnect::core::{CoreIdToken, CoreIdTokenClaims};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::sync::Mutex;

pub const SESSION_ID: &str = "session_id";
pub struct SessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    Unauthenticated,
    Authenticated(AuthenticatedUser),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub subject: String,
    pub name: String,
    pub roles: HashSet<Role>,
}

impl AuthenticatedUser {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|role| self.roles.contains(role))
    }

    /// Check if user has all the specified roles
    pub fn has_all_roles(&self, roles: &[Role]) -> bool {
        roles.iter().all(|role| self.roles.contains(role))
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }

}
impl fmt::Display for AuthenticatedUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "User(sub:{}, name:{}, roles:{:?})", self.subject,self.name,self.roles)
    }
}
#[cfg(feature = "ssr")]
fn extract_auth_user(session: &SessionData, claims: &CoreIdTokenClaims) -> Result<AuthenticatedUser, Error> {
    use leptos::serde_json;
    use oauth2::{RefreshToken, TokenResponse};
    use openidconnect::core::CoreIdToken;
    use std::str::FromStr;
    use tracing::info;

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

#[cfg(feature = "ssr")]
impl FromRequestParts<AppState> for AuthenticatedUser
where
    Self: 'static,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        use axum_extra::extract::CookieJar;
        use leptos::serde_json;
        use oauth2::{RefreshToken, TokenResponse};
        use openidconnect::core::CoreIdToken;
        use std::str::FromStr;
        use tracing::info;
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
#[cfg(feature = "ssr")]
impl TryFrom<&SessionData> for Auth {
    type Error = &'static str;

    fn try_from(data: &SessionData) -> Result<Self, Self::Error> {
        use leptos::serde_json;
        use oauth2::{RefreshToken, TokenResponse};
        use openidconnect::core::CoreIdToken;
        use std::str::FromStr;
        use tracing::info;

        if let Some(id_token_str) = &data.id_token
            && let Some(id_token) = CoreIdToken::from_str(id_token_str).ok()
            && let Some(name) = &data.name
            && let Some(subject) = &data.subject
        {
            let user = AuthenticatedUser {
                subject: subject.clone(),
                name: name.clone(),
                roles: data.roles.clone()
            };
            Ok(Auth::Authenticated(user))
        } else {
            Ok(Auth::Unauthenticated)
        }
    }
}
#[cfg(feature = "ssr")]
impl FromRequestParts<AppState> for SessionId
where
    Self: 'static,
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let session_cookie = headers
            .get(http::header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split(';').find_map(|cookie_str| {
                    if let Ok(cookie) = Cookie::parse(cookie_str.trim())
                        && cookie.name() == SESSION_ID {
                            return Some(cookie.value().to_owned());
                        }
                    None
                })
            });

        if let Some(id) = session_cookie {
            Ok(SessionId(id))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Session cookie not found"))
        }
    }
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
