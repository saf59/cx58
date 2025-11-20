
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use gm::state::AppState;
    use axum::{routing::get, Router}; //post
    use gm::{app::*, ssr::*};
    use leptos_axum::file_and_error_handler;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tokio::net::TcpListener;
    use tower_cookies::CookieManagerLayer;
    use tracing::info;
    use axum::extract::Request;
    use axum::response::Response;
    use axum::middleware::Next;
    use axum::middleware;

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    let state = AppState::init().await.unwrap();
    let leptos_routes = generate_route_list(App);

    let app = Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/logout", get(logout_handler))
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/{*fn_name}", get( leptos_server_fn_handler).post( leptos_server_fn_handler))
        //.nest_service("/pkg", ServeDir::new("/pkg"))
        .leptos_routes_with_handler(leptos_routes, leptos_main_handler)
        .layer(middleware::from_fn(log_uri))
        .fallback(file_and_error_handler::<AppState, _>(shell))
        .layer(axum::extract::Extension(state.clone()))
        .layer(CookieManagerLayer::new())
        .with_state(state.clone());
    async fn log_uri(req: Request, next: Next) -> Response {
        tracing::info!("[{}] {}", req.method(), req.uri());
        next.run(req).await
    }
    let listener = TcpListener::bind(state.leptos_options.site_addr).await.unwrap();
    info!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );
    //info!("{:#?}",&leptos_routes);
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
