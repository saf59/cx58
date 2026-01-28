use crate::auth::Auth;
use leptos::prelude::ServerFnError;
use leptos::logging::error;
use leptos_macro::server;
#[allow(unused_imports)]
use tracing::info;

#[server(GetAuth, "/api")]
pub async fn get_auth() -> Result<Auth, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let auth = match leptos::context::use_context::<Auth>() {
            Some(a) => a,
            None => {
                return Err(ServerFnError::ServerError(
                    "User not authenticated (Auth context missing).".into(),
                ));
            }
        };
        Ok(auth)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}

/*#[server(GetMediaProxy, "/api")]
pub async fn get_media_proxy() -> Result<String, ServerFnError> {
    let media_proxy =  match leptos::context::use_context::<ClientConfig>() {
        Some(config) => config.media_proxy,
        None=> {
            error!("GetMediaProxy: ClientConfig context missing.");
            return Err(ServerFnError::ServerError(
                "GetMediaProxy: ClientConfig context missing.".into(),
            ));
        }
    };
    Ok(media_proxy)
}*/

/*#[server(GetMediaProxy, "/api")]
pub async fn get_media_proxy() -> Result<String, ServerFnError> {
    use crate::state::AppState;
    let app_state = leptos::context::use_context::<AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found"))?;
    Ok(app_state.oidc_client.config.media_proxy.clone())
}*/