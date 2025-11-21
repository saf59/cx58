use crate::auth::*;
use crate::state::AppState;
use axum::{
    Error,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Redirect, Response},
};

use axum_extra::extract::{CookieJar, cookie::Cookie};
use leptos::serde_json;
use oauth2::{CsrfToken, PkceCodeVerifier, RefreshToken, TokenResponse};
use openidconnect::Nonce;
use openidconnect::core::{CoreIdToken, CoreIdTokenClaims};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
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
// Время, за которое до истечения токена начинаем обновление (5 минут)
const REFRESH_THRESHOLD: Duration = Duration::from_secs(5 * 60);

// Тип для удобного извлечения (может быть и просто SessionData)
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
// -------------------------------------------------------------------------
// 💡 Вспомогательные функции, вынесенные из from_request_parts
// -------------------------------------------------------------------------

/// Извлекает ID сессии из заголовков Cookie.
fn get_session_id_from_parts(parts: &Parts) -> Result<String, Box<Response>> {

    // Получение заголовка Cookie
    let cookie_header_value = parts.headers
        .get(http::header::COOKIE)
        .and_then(|h| h.to_str().ok());

    let session_id = cookie_header_value
        .and_then(|cookies_str| {
            // Ищем куку с нужным именем
            cookies_str.split(';')
                .find_map(|cookie| {
                    let mut parts = cookie.trim().split('=');
                    // Проверяем, что есть имя и значение
                    let name = parts.next()?;
                    let value = parts.next()?;

                    if name == SESSION_ID {
                        // Возвращаем значение
                        Some(value.to_string())
                    } else {
                        None
                    }
                })
        });

    // 2. Обработка ошибки, оборачивая Response в Box
    match session_id {
        Some(id) => Ok(id),
        None => {
            let response = (StatusCode::UNAUTHORIZED, "Missing or invalid session ID cookie.").into_response();
            Err(Box::new(response))
        }
    }
}

/// Асинхронно сохраняет обновленные данные сессии в HashMap.
async fn save_session_data_in_store(
    store: &Arc<Mutex<HashMap<String, SessionData>>>,
    session_id: String,
    data: SessionData,
) {
    let mut map_guard = store.lock().await;
    map_guard.insert(session_id, data);
}

// Предполагаем, что REFRESH_THRESHOLD и SESSION_ID определены где-то как константы
// const REFRESH_THRESHOLD: std::time::Duration = ...;
// const SESSION_ID: &str = ...;

/// Эта функция возвращает SessionData, если сессия валидна (не истек Hard Expiration).
/// Она также запускает фоновое обновление токена, если подошло время (Soft Expiration).
pub async fn get_and_refresh_session(
    state: &AppState,
    session_id: &str,
) -> Option<SessionData> {
    // 1. Получаем данные сессии (клонируем, чтобы отпустить мьютекс)
    let session_data = {
        let sessions = state.sessions.lock().await;
        sessions.get(session_id).cloned()? // Если нет в базе -> None
    };

    let now = Instant::now();

    // 2. Проверка Hard Expiration
    // Если токен протух окончательно -> возвращаем None (пользователь разлогинен)
    if session_data.id_token_expires_at.is_some_and(|exp_at| now >= exp_at) {
        return None;
    }

    // 3. Проверка необходимости обновления (Refresh Threshold)
    let needs_refresh = session_data.id_token_expires_at.is_some_and(|exp_at| {
        exp_at.checked_sub(REFRESH_THRESHOLD).is_some_and(|future_time| {
            trace_time("Refresh time at",&Some(future_time));
            future_time <= now
        })
    });

    // 4. Запуск фонового обновления
    if needs_refresh && session_data.refresh_token.is_some() {
        // Проверяем флаг is_refreshing без блокирования всей мапы сессий
        let mut is_refreshing_lock = session_data.is_refreshing.lock().await;

        if !*is_refreshing_lock {
            *is_refreshing_lock = true;
            drop(is_refreshing_lock); // Важно: освобождаем lock перед спавном

            let refresh_token = session_data.refresh_token.clone().unwrap();
            let session_id_clone = session_id.to_owned();

            // Клонируем зависимости для задней задачи
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
                        // Шаг 1: Обновляем в памяти и получаем копию
                        let updated_session = {
                            let mut guard = session_store_clone.lock().await;
                            if let Some(current_data) = guard.get_mut(&session_id_clone) {
                                current_data.id_token = Some(new_id_token);
                                current_data.refresh_token = new_refresh_token;
                                current_data.id_token_expires_at = Some(new_expires_at);
                                *current_data.is_refreshing.lock().await = false;

                                trace_time("Updated session ID Token expires at",&current_data.id_token_expires_at);

                                // Возвращаем клон обновленных данных
                                Some(current_data.clone())
                            } else {
                                None
                            }
                        }; // <--- Здесь guard уничтожается, и лок освобождается

                        // Шаг 2: Сохраняем (в БД/Redis или просто перезаписываем, если функция требует этого)
                        if let Some(data) = updated_session {
                            save_session_data_in_store(&session_store_clone, session_id_clone, data).await;
                        }
                    },

                    Err(e) => {
                        tracing::error!("Token refresh failed for session {}: {:?}", session_id_clone, e);
                        // Сбрасываем флаг при ошибке
                        let guard = session_store_clone.lock().await;
                        if let Some(current_data) = guard.get(&session_id_clone) {
                            *current_data.is_refreshing.lock().await = false;
                        }
                    }
                }
            });
        }
    }

    // 5. Возвращаем текущие данные (даже если обновление запущено, возвращаем старые валидные данные)
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

// -------------------------------------------------------------------------
// 🚀 Структура и Реализация FromRequestParts
// -------------------------------------------------------------------------

impl FromRequestParts<AppState> for SessionDataWithRefresh
where
// Требование Self: 'static не нужно, так как оно уже неявно
// обеспечивается FromRequestParts и асинхронными блоками.
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        tracing::info!("FromRequestParts<AppState> for SessionDataWithRefresh");
        // --- 1. Извлечение Session ID и данных ---

        let session_id = get_session_id_from_parts(parts).unwrap();

        // Получаем клонированные данные сессии
        let session_data: SessionData = {
            let map_guard = state.sessions.lock().await;
            map_guard.get(&session_id)
                .cloned()
                .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Session not found in store.").into_response())?
        };

        let now = Instant::now();

        // --- 2. Проверка Hard Expiration ---

        if session_data.id_token_expires_at.is_some_and(|exp_at| now >= exp_at) {
            return Err((StatusCode::UNAUTHORIZED, "ID Token Expired. Please log in again.").into_response());
        }

        // --- 3. Проверка необходимости обновления (Refresh Threshold) ---
/*        let needs_refresh = session_data.id_token_expires_at.map_or(false, |exp_at| {
            exp_at.checked_sub(REFRESH_THRESHOLD).map_or(false, |threshold_time| {
                now >= threshold_time
            })
        });
*/
        // Вместо вычитания из exp_at, мы прибавляем REFRESH_THRESHOLD к now
        let needs_refresh = session_data.id_token_expires_at.is_some_and( |exp_at| {
            now.checked_add(REFRESH_THRESHOLD).is_some_and(|future_time| {
                // Если (текущее время + порог) >= время истечения
                // значит, до истечения осталось меньше, чем порог.
                future_time >= exp_at
            })
        });
        // --- 4. Запуск фонового обновления, если необходимо ---

        if needs_refresh && session_data.refresh_token.is_some() {
            let mut is_refreshing_lock = session_data.is_refreshing.lock().await;

            if !*is_refreshing_lock {
                *is_refreshing_lock = true;
                drop(is_refreshing_lock); // Освобождаем мьютекс

                let refresh_token = session_data.refresh_token.clone().unwrap();
                let session_id_clone = session_id.clone();

                // Клонирование глобальных зависимостей (остаются прежними)
                let session_store_clone = state.sessions.clone();
                let oidc_client_clone = state.oidc_client.clone();
                let http_client_clone = state.async_http_client.clone();

                tokio::spawn(async move {

                    match perform_token_refresh(
                        refresh_token,
                        &oidc_client_clone,
                        &http_client_clone
                    ).await {
                        Ok((new_id_token, new_refresh_token, new_expires_at)) => {

                            // Получаем данные, обновляем и сохраняем
                            if let Some(mut current_data) = {
                                let guard = session_store_clone.lock().await;
                                guard.get(&session_id_clone).cloned()
                            } {
                                current_data.id_token = Some(new_id_token);
                                current_data.refresh_token = new_refresh_token;
                                current_data.id_token_expires_at = Some(new_expires_at);

                                *current_data.is_refreshing.lock().await = false;

                                save_session_data_in_store(&session_store_clone, session_id_clone, current_data).await;
                            }
                        },
                        Err(e) => {
                            tracing::error!("Token refresh failed for session {}: {:?}", session_id_clone, e);

                            // ⚠️ Очищаем флаг, чтобы не блокировать будущие попытки.
                            if let Some( current_data) = {
                                let guard = session_store_clone.lock().await;
                                guard.get(&session_id_clone).cloned()
                            } {
                                *current_data.is_refreshing.lock().await = false;
                                save_session_data_in_store(&session_store_clone, session_id_clone, current_data).await;
                            }
                        }
                    }
                });
            }
        }

        // 5. Возвращаем текущие данные сессии
        Ok(SessionDataWithRefresh(session_data))
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
                        && cookie.name() == SESSION_ID
                    {
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

type RefreshResult =
    Result<(String, Option<String>, Instant), Box<dyn std::error::Error + Send + Sync>>;

/// Выполняет запрос на обновление токена и валидирует новый ID Token.
pub async fn perform_token_refresh(
    current_refresh_token: String,
    oidc_client: &crate::ssr::ISPOidcClient,
    http_client: &reqwest::Client,
) -> RefreshResult {
    let refresh_token = RefreshToken::new(current_refresh_token);

    // 1. Выполнение запроса на обновление токена
    let token_response = oidc_client
        .exchange_refresh_token(&refresh_token,http_client)
        .await
        .map_err(|e| format!("Refresh token request failed: {:?}", e))?;

    // 2. Получение ID Token и его валидация
    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or("Missing ID Token after refresh")?;

    let claims = id_token
        .claims(&oidc_client.id_token_verifier(), |_nonce: Option<&openidconnect::Nonce>| -> Result<(), String> {
            Ok(())
        },)
        .map_err(|e| format!("ID Token validation failed after refresh: {:?}", e))?;

    // 3. Извлечение нового срока действия (exp)
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

    // 4. Возврат данных
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
