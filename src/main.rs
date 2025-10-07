#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use cx58::auth::{auth_callback, logout_handler, AppState, AuthTokenLayer};
    use cx58::app::*;
    use cx58::config::AppConfig;
    // 1. Создаем ПОЛНУЮ конфигурацию один раз
    let app_config = AppConfig::from_env().expect("Failed to load config");
    let app_config_clone = app_config.clone();
    // 2. Получаем стандартную конфигурацию Leptos
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    // 3. Создаем единое состояние приложения
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        config: app_config_clone,
    };
    // 4. Собираем Router
    let app = Router::new()
        .leptos_routes(&app_state, routes, {
            let leptos_options = app_state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .route("/api/auth/callback", axum::routing::get(auth_callback))
        .route("/api/auth/logout", axum::routing::post(logout_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(axum::Extension(app_config.clone()))
        .layer(AuthTokenLayer::new(app_config))
        .with_state(app_state);
    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}

