use crate::auth::Auth;
use leptos::prelude::ServerFnError;
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
