#[cfg(feature = "ssr")]
use crate::state::AppState;
use leptos::prelude::ServerFnError;
use leptos_macro::server;
#[allow(unused_imports)]
use tracing::info;
use crate::auth::*;

#[server(GetAuth, "/api")]
pub async fn get_auth() -> Result<AuthenticatedUser, ServerFnError> {
    #[cfg(feature = "ssr")]
    #[allow(unused_must_use)]
    {
        use leptos_axum::extract_with_state;
        use leptos::context::use_context;

        let state = match use_context::<AppState>() {
            Some(s) => s,
            None => {
                return Err(ServerFnError::ServerError(
                    "Missing AppState context".into(),
                ));
            }
        };

        let user = match extract_with_state::<AuthenticatedUser, AppState>(&state).await {
            Ok(user) => user,
            Err(rejection) => {
                return Err(ServerFnError::ServerError(format!(
                    "Authentication required: {:?}",
                    rejection
                )));
            }
        };

        Ok(user)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}

#[server(GetIsAuthenticated, "/api")]
pub async fn get_is_authenticated() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    #[allow(unused_must_use)]
    {
        use leptos::context::use_context;
        use leptos_axum::extract;
        use axum_extra::extract::CookieJar;

        let jar = match extract::<CookieJar>().await {
            Ok(jar) => jar,
            Err(_) => {
                return Err(ServerFnError::ServerError("Authentication context missing.".to_string(),));
            }
        };
        let is_authenticated = if let Some(cookie) = jar.get(SESSION_ID) {
            let session_id = cookie.value().to_string();
            let state = use_context::<AppState>()
                .expect("AppState not found in context.");
            state.sessions.lock().await.contains_key(&session_id)
        } else {
            false
        };

        Ok(is_authenticated)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}

