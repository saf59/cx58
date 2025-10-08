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
    use base64::Engine;
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
    // Generate the list of routes in the Leptos App
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

    async fn security_headers(mut req: Request, next: Next) -> axum::response::Response {
        // Detect environment early
        let app_env = std::env::var("LEPTOS_ENV").ok()
            .or_else(|| std::env::var("APP_ENV").ok())
            .unwrap_or_else(|| "DEV".to_string());
        let is_prod = matches!(app_env.as_str(), "PROD" | "prod" | "Production" | "production");

        // Generate per-request CSP nonce and place into request extensions for SSR usage
        let mut nonce_bytes = [0u8; 16];
        let _ = getrandom::getrandom(&mut nonce_bytes);
        let nonce = base64::engine::general_purpose::STANDARD_NO_PAD.encode(nonce_bytes);
        req.extensions_mut().insert(nonce.clone());

        // Continue the pipeline
        let mut res = next.run(req).await;
        let headers = res.headers_mut();

        // Core security headers
        let _ = headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );
        let _ = headers.insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        );
        let _ = headers.insert(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        );
        let _ = headers.insert(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        );
        let _ = headers.insert(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        );
        let _ = headers.insert(
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderValue::from_static("same-origin"),
        );
        let _ = headers.insert(
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        );

        // HSTS only in PROD (assumes HTTPS termination in front)
        if is_prod {
            let _ = headers.insert(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );
        }

        // Expose nonce for potential client-side usage (optional)
        if let Ok(val) = HeaderValue::from_str(&nonce) {
            let _ = headers.insert(HeaderName::from_static("x-csp-nonce"), val);
        }

        // Content-Security-Policy
        // DEV: relaxed to support HMR/hydration; include nonce as we prepare to remove 'unsafe-inline' later
        // PROD: no ws/wss and no 'unsafe-eval'; keep 'unsafe-inline' for styles, include nonce for scripts/styles
        let csp = if !is_prod {
            format!(
                "default-src 'self'; frame-ancestors 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' 'nonce-{}'; style-src 'self' 'unsafe-inline' https://use.fontawesome.com 'nonce-{}'; img-src 'self' data: blob:; font-src 'self' data: https://use.fontawesome.com; connect-src 'self' ws: wss:",
                nonce, nonce
            )
        } else {
            format!(
                "default-src 'self'; frame-ancestors 'none'; script-src 'self' 'nonce-{}' 'wasm-unsafe-eval'; style-src 'self' https://use.fontawesome.com 'nonce-{}'; img-src 'self' data: blob:; font-src 'self' data: https://use.fontawesome.com; connect-src 'self'",
                nonce, nonce
            )
        };

        if let Ok(val) = HeaderValue::from_str(&csp) {
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

