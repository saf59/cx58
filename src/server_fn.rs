use crate::auth::Auth;
use leptos::prelude::ServerFnError;
use leptos_macro::server;
#[allow(unused_imports)]
use tracing::info;

#[server(GetAuth, "/api")]
pub async fn get_auth() -> Result<Auth, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use leptos::context::use_context;
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
        use leptos::context::use_context;
        match use_context::<Auth>() {
            Some(auth) => match auth {
                Auth::Authenticated(_) => Ok(true),
                Auth::Unauthenticated => Ok(false),
            },
            None => Err(ServerFnError::ServerError(
                "User not authenticated (Auth context missing).".into(),
            )),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}
/*
#[server]

pub async fn check_and_refresh_session() -> Result<(), ServerFnError<String>> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;
        use axum::extract::State;
        use crate::auth_ssr::*;
        use leptos_axum::extract_with_state;
        use leptos_axum::extract;
        use leptos::prelude::expect_context;
        // Внимание: Этот код выполняется ТОЛЬКО на сервере (Axum).

        // 1. Извлечение AppState и SessionDataWithRefresh
        // Axum-маршрутизатор настроен так, чтобы предоставить AppState и
        // запустить FromRequestParts для SessionDataWithRefresh.
        //
        // Здесь мы используем ServerFn-синтаксис для извлечения.
        // **Ключевой момент:** Извлечение SessionDataWithRefresh здесь
        // автоматически запустит логику проверки и, при необходимости,
        // асинхронное фоновое обновление токенов, реализованное в
        // вашем FromRequestParts.
        tracing::info!("check_and_refresh_session");
       // let state_extractor  = extract::<axum::extract::State<crate::state::AppState>>().await
       //     .map_err(|e| ServerFnError::ServerError(format!("Failed to extract State<AppState>: {:?}", e)))?;

        // Получаем доступ к самой структуре AppState через .0
       // let state = state_extractor.0;

        let state: AppState = extract::<AppState>().await
            .map_err(|e| ServerFnError::ServerError(format!("Failed to extract AppState: {:?}", e)))?;

        //match axum::extract::State::<AppState>::from_request_parts(&mut parts, &state).await {
        //match extract::<AppState>().await {
        //    Ok(State(app_state)) => {
        // Если вам нужен сам AppState, вы можете извлечь его.
        // Однако, для запуска экстрактора SessionDataWithRefresh,
        // нам нужно его извлечь:
        //match SessionDataWithRefresh::from_request_parts(&mut parts, &state).await {
        match extract_with_state::<SessionDataWithRefresh, AppState>(&state).await {
            //match extract::<SessionDataWithRefresh>().await {
            Ok(SessionDataWithRefresh(_session_data)) => {
                // Сессия извлечена. Проверка expired и запуск
                // обновления произошли в FromRequestParts.
                // Теперь просто возвращаем Ok.
                Ok(())
            }
            Err(e) => Err(ServerFnError::ServerError(format!("Failed to extract SessionDataWithRefresh: {:?}", e)))
            /*                    Err((status, msg)) if status == StatusCode::UNAUTHORIZED => {
                                    // Токен окончательно истек (Hard Expiration).
                                    // Возвращаем ошибку, которая на фронте может
                                    // вызвать редирект на логин.
                                    Err(ServerFnError::ServerError(format!(
                                        "Session expired: {}",
                                        msg
                                    )))
                                }
                                Err((_, msg)) => Err(ServerFnError::ServerError(format!(
                                    "Session check failed: {}",
                                    msg
                                ))),
                            }
                        }
                        Err(e) => Err(ServerFnError::ServerError(format!(
                            "Failed to extract AppState: {:?}",
                            e
                        ))),
                    }
            */
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::<()>::ServerError(
            "Cannot run ServerFn on client.".to_string(),
        ))
    }
}
*/