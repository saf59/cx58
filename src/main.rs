#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::get, Router, middleware};
    use axum::http::{HeaderName, HeaderValue};
    use axum::extract::Request;
    use axum::middleware::Next;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use cx58::auth::{auth_callback, logout_handler, AppState, AuthTokenLayer };
    use cx58::app::*;
    use cx58::config::AppConfig;
    use cx58::rbac::{ensure_role, Authenticated, Role};
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
    async fn me(Authenticated(claims): Authenticated) -> String {
        format!("hello {}", claims.name.unwrap_or(claims.sub))
    }

    async fn admin_stats(Authenticated(claims): Authenticated) -> Result<&'static str, (axum::http::StatusCode, &'static str)> {
        ensure_role(&claims, Role::Admin)?;
        Ok("Top secret stats")
    }

    async fn security_headers(req: Request, next: Next) -> axum::response::Response {
        let mut res = next.run(req).await;
        let headers = res.headers_mut();
        // X-Frame-Options
        let _ = headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );
        // X-Content-Type-Options
        let _ = headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );
        // X-XSS-Protection (legacy; some scanners still require it)
        let _ = headers.insert(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        );
        // Content-Security-Policy (adjust as needed for your app)
        // Note: this CSP is relaxed for DEV to avoid breaking hot-reload and external fonts.
        let csp = "default-src 'self'; frame-ancestors 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://use.fontawesome.com; img-src 'self' data: blob:; font-src 'self' data: https://use.fontawesome.com; connect-src 'self' ws: wss:";
        if let Ok(val) = HeaderValue::from_str(csp) {
            let _ = headers.insert(HeaderName::from_static("content-security-policy"), val);
        }
        res
    }

    let app = Router::new()
        .leptos_routes(&app_state, routes, {
            let leptos_options = app_state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .route("/api/auth/callback", axum::routing::get(auth_callback))
        .route("/api/auth/logout", axum::routing::post(logout_handler))
        .route("/api/me", get(me))
        .route("/api/admin/stats", get(admin_stats))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(middleware::from_fn(security_headers))
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

