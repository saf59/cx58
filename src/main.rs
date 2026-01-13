#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::middleware;
    use axum::{
        Router,
        routing::{get, post},
    }; //post
    use gmr::stop::stop_handler;
    use gmr::proxy_tree::proxy_tree_handler;
    use gmr::{app::*, llm_stream::*, ssr::*, state::AppState};
    use leptos_axum::file_and_error_handler;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tokio::net::TcpListener;
    use tower_cookies::CookieManagerLayer;
    use tracing::info;

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let leptos_routes = generate_route_list(App);
    let state = AppState::init().await.unwrap();

    let app = Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/logout", get(logout_handler))
        .route("/api/health", get(|| async { "OK" }))
        .route(
            "/api/get_auth{_}",
            post(leptos_server_fn_handler).get(leptos_server_fn_handler),
        )
        .route("/api/stop", post(stop_handler))
        .route("/api/proxy/tree/{user_id}", get(proxy_tree_handler))
        .route("/api/chat_stream", axum::routing::post(chat_stream_handler))
        .leptos_routes_with_handler(leptos_routes.clone(), leptos_main_handler)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        //.layer(axum::middleware::from_fn(log_uri))
        .fallback(file_and_error_handler::<AppState, _>(shell))
        .layer(axum::extract::Extension(state.clone()))
        .layer(CookieManagerLayer::new())
        .with_state(state.clone());
    //.layer(tower_http::compression::CompressionLayerCompressionLayer::new().gzip(true));

    let listener = TcpListener::bind(state.leptos_options.site_addr)
        .await
        .unwrap();
    info!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );
    //info!("{:#?}", &leptos_routes);
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
