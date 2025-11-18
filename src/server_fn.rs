use crate::auth::Auth;
use leptos::context::use_context;
use leptos::prelude::ServerFnError;
use leptos_macro::server;
#[allow(unused_imports)]
use tracing::info;

#[server(GetAuth, "/api")]
pub async fn get_auth() -> Result<Auth, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let auth = match use_context::<Auth>() {
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

#[server(GetIsAuthenticated, "/api")]
pub async fn get_is_authenticated() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match use_context::<Auth>() {
            Some(auth) => match auth {
                Auth::Authenticated(_) => Ok(true),
                Auth::Unauthenticated => Ok(false)
            },
            None => {
                Err(ServerFnError::ServerError(
                    "User not authenticated (Auth context missing).".into(),
                ))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}
