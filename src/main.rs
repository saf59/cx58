#![allow(unused_imports)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::routing::*;
    use axum::{
        body::Body,
        extract::State,
        http::{HeaderValue, Request},
        middleware::{self, Next},
        response::{IntoResponse, Json},
        Router,
    };
    use axum_extra::extract::CookieJar;
    use leptos_axum::{
        file_and_error_handler, generate_route_list
        , LeptosRoutes,
    };
    use leptos_oidc::Auth;
    use std::sync::Arc;
    //use axum::http::{HeaderName, HeaderValue};
    use axum::Extension;
    use base64::Engine;
    use cx58::app::*;
    use cx58::auth::{start_login, auth_callback, logout, AppState, AuthTokenLayer};
    use cx58::config::AppConfig;
    use cx58::rbac::{ensure_role, Authenticated, Role};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_oidc::AuthSignal;
    let app_config = AppConfig::from_env().expect("Failed to load config");
    let session_store =
        RedisSessionStore::new("redis://127.0.0.1:6379").expect("Redis must be running");
    let app_config_clone = app_config.clone();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        config: app_config_clone,
        session_store: Arc::new(session_store),
    };
    use async_redis_session_v2::RedisSessionStore;
    /// --- 4. Router setup
    pub fn create_router(app_config: AppConfig, app_state: AppState) -> Router {
        let app = App;
        let routes = generate_route_list(app);
        let arc_app_config = Arc::new(app_config.clone());
        // ---- API router (no CSP needed)
        let api_router = Router::new()
            .route("/api/start_login", get(start_login))
            .route("/api/auth/callback", get(auth_callback))
            //.route("/api/auth/logout", post(logout_handler))
            .route("/api/logout", post(logout))
            .route("/api/me", get(me))
            .route("/api/admin/stats", get(admin_stats))
            .route("/api/health", get(|| async { "OK" }))
            .route(
                "/api/get_auth_parameters",
                get(Json(app_config.auth_parameters())),
            )
            .route(
                "/api/get_auth_parameters{_}",
                get(Json(app_config.auth_parameters())),
            );

        // ---- Leptos router (CSP applied)
        let leptos_router = Router::new()
            .leptos_routes_with_handler(routes, get(leptos_handler))
            //.leptos_routes(&app_state, routes, app)
            .fallback(file_and_error_handler::<AppState, _>(shell))
            // ✅ apply CSP only to Leptos-rendered routes
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                security_headers,
            ));

        // ---- Merge routers together
        Router::new()
            .merge(api_router)
            .merge(leptos_router)
            .layer(Extension(arc_app_config))
            .layer(AuthTokenLayer::new(app_config))
            .with_state(app_state)
    }
    // ---------- Leptos SSR handler ----------
    async fn leptos_handler(
        State(app_state): State<AppState>,
        req: axum_core::extract::Request<axum::body::Body>,
    ) -> impl IntoResponse {
        let nonce = req
            .extensions()
            .get::<Nonce>()
            .cloned()
            .unwrap_or_else(Nonce::new);
        let leptos_options = app_state.leptos_options.clone();
        let leptos_options_clone = leptos_options.clone();
        let auth_parameters = app_state.config.auth_parameters();

        let handler = leptos_axum::render_app_to_stream_with_context(
            move || {
                let auth_signal: AuthSignal = Auth::signal();
                provide_context(auth_signal);
                let _ = Auth::init(auth_parameters.clone());
                provide_context(app_state.config.clone());
                provide_context(leptos_options.clone());
                provide_context(nonce.clone());
            },
            move || shell(leptos_options_clone.clone()),
        );
        handler(req).await.into_response()
    }

    async fn me(Authenticated(claims): Authenticated) -> String {
        format!("hello {}", claims.name.unwrap_or(claims.sub))
    }

    async fn admin_stats(
        Authenticated(claims): Authenticated,
    ) -> Result<&'static str, (axum::http::StatusCode, &'static str)> {
        ensure_role(&claims, Role::Admin)?;
        Ok("Top secret stats")
    }
    /// Middleware, with CSP 3 && security headers
    /// && nonce in Request Extensions, for Leptos
    pub async fn security_headers(
        State(app_state): State<AppState>,
        mut req: Request<Body>,
        next: Next,
    ) -> impl IntoResponse {
        println!(">>> security_headers called for {}", req.uri());
        let uri = req.uri().path().to_string();
        // ❌ We do not add CSP for static or API
        if uri.starts_with("/pkg")
            || uri.starts_with("/assets")
            || uri.starts_with("/api")
            || uri.ends_with(".js")
            || uri.ends_with(".css")
            || uri.ends_with(".wasm")
            || uri.ends_with(".map")
            || uri.ends_with(".ico")
        {
            return next.run(req).await;
        }

        let is_prod = app_state.config.is_prod;
        let trust_data_list = app_state.config.trust_data_list;
        let trust_connect_list = app_state.config.trust_connect_list;
        let nonce = Nonce::new();
        req.extensions_mut().insert(nonce.clone());
        let mut res = next.run(req).await;
        let nonce = nonce.to_string();

        // Content-Security-Policy
        let csp = if !is_prod {
            // DEV: relaxed to support HMR/hydration;
            // include nonce as we prepare to remove 'unsafe-inline' later
            format!(
                "default-src 'self'; \
                frame-ancestors 'none'; \
                script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' 'nonce-{}'; \
                style-src 'self' 'unsafe-inline' {trust_data_list} 'nonce-{}'; \
                img-src 'self' data: blob:; \
                font-src 'self' data: {trust_data_list}; \
                connect-src 'self' ws: wss: {trust_connect_list}",
                nonce, nonce
            )
        } else {
            // PROD: no ws/wss and no 'unsafe-eval'; keep 'unsafe-inline' for styles,
            // include nonce for scripts/styles
            format!(
                "default-src 'self';\
                 frame-ancestors 'none'; \
                 script-src 'self' 'nonce-{}' 'wasm-unsafe-eval'; \
                 style-src 'self' {trust_data_list} 'nonce-{}'; \
                 img-src 'self' data: blob:; \
                 font-src 'self' data: {trust_data_list}; \
                 connect-src 'self' {trust_connect_list}",
                nonce, nonce
            )
        };
        //println!("{csp}");
        let headers = res.headers_mut();
        headers.insert(
            "Content-Security-Policy",
            HeaderValue::from_str(&csp).unwrap(),
        );
        headers.insert(
            "X-Content-Type-Options",
            HeaderValue::from_static("nosniff"),
        );
        headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
        headers.insert("Referrer-Policy", HeaderValue::from_static("no-referrer"));
        headers.insert(
            "Cross-Origin-Embedder-Policy",
            HeaderValue::from_static("require-corp"),
        );
        headers.insert(
            "Cross-Origin-Opener-Policy",
            HeaderValue::from_static("same-origin"),
        );
        headers.insert(
            "Cross-Origin-Resource-Policy",
            HeaderValue::from_static("same-origin"),
        );

        headers.insert(
            "X-XSS-Protection",
            HeaderValue::from_static("1; mode=block"),
        );
        headers.insert(
            "Permissions-Policy",
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        );
        // HSTS only in PROD (assumes HTTPS termination in front)
        if is_prod {
            headers.insert(
                "Strict-Transport-Security",
                HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
            );
        }

        res
    }

    let app = create_router(app_config, app_state);
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
