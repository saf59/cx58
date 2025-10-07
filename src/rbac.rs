#![allow(unused_imports)]
#[cfg(feature = "ssr")]
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
#[cfg(feature = "ssr")]
use crate::auth::Claims;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
}

#[cfg(feature = "ssr")]
impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
        }
    }
}
#[cfg(feature = "ssr")]
impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "admin" | "Admin" => Role::Admin,
            _ => Role::User,
        }
    }
}

#[cfg(feature = "ssr")]
pub fn has_role(claims: &Claims, role: Role) -> bool {
    claims
        .roles
        .as_ref()
        .map(|rs| rs.iter().any(|r| r.eq_ignore_ascii_case(role.as_str())))
        .unwrap_or(false)
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
pub struct Authenticated(pub Claims);

#[cfg(feature = "ssr")]
impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Claims are injected by AuthTokenMiddleware into request extensions if a valid token is present
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(Authenticated(claims.clone()));
        }
        Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}

#[cfg(feature = "ssr")]
pub fn ensure_role(claims: &Claims, role: Role) -> Result<(), (StatusCode, &'static str)> {
    if has_role(claims, role) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Forbidden"))
    }
}
