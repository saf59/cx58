#![allow(unused_imports)]

use leptos_oidc::{Auth, AuthParameters};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::routing::*;
    use std::sync::Arc;
    use axum::{
        body::Body,
        extract::State,
        http::{HeaderMap, Request,HeaderName, HeaderValue,header::CONTENT_SECURITY_POLICY},
        middleware::{self, Next},
        response::{IntoResponse,Json},
        routing::get,
        Router,
    };
    use axum_extra::routing::RouterExt;
    use leptos::*;
    use leptos_axum::{
        generate_route_list, render_app_to_stream_with_context, file_and_error_handler,
        handle_server_fns_with_context,
        LeptosRoutes, ResponseOptions,
    };
    //use axum::http::{HeaderName, HeaderValue};
    use axum::Extension;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_oidc::AuthSignal;
    use base64::Engine;
    use cx58::auth::{auth_callback, logout_handler, AppState, AuthTokenLayer};
    use cx58::app::*;
    use cx58::config::AppConfig;
    use cx58::rbac::{ensure_role, Authenticated, Role};
    let app_config = AppConfig::from_env().expect("Failed to load config");
    let app_config_clone = app_config.clone();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        config: app_config_clone,
    };

    /// --- 4. Router setup
    pub fn create_router(app_config: AppConfig, app_state: AppState) -> Router {
        let app = App;
        let routes = generate_route_list(app);

        // ---- API router (no CSP needed)
        let api_router = Router::new()
            .route("/api/auth/callback", get(auth_callback))
            .route("/api/auth/logout", post(logout_handler))
            .route("/api/me", get(me))
            .route("/api/admin/stats", get(admin_stats))
            .route("/api/health", get(|| async { "OK" }))
            //.route("/api/get_auth_parameters/*", get(Json(app_config.auth_parameters())))
            .route_with_tsr("/api/get_auth_parameters{_rand}", get(Json(app_config.auth_parameters())))
/*            .handle_server_fns_with_context(app_config.clone(), move |cx| {
                provide_context(app_state.clone());
            })
*/
            .layer(Extension(app_config.clone()))
            .layer(AuthTokenLayer::new(app_config));

        // ---- Leptos router (CSP applied)
        let leptos_router = Router::new()
            .leptos_routes_with_handler(routes, get(leptos_handler))
            //.leptos_routes(&app_state, routes, app)
            .fallback(file_and_error_handler::<AppState,_>(shell))
            // ✅ apply CSP only to Leptos-rendered routes
            .layer(middleware::from_fn_with_state(app_state.clone(), security_headers));

        // ---- Merge routers together
        Router::new()
            .merge(api_router)
            .merge(leptos_router)
            .with_state(app_state)
    }
/*    pub async fn leptos_routes_handler(
        auth_session: AuthSession,
        State(app_state): State<AppState>,
        axum::extract::State(option): axum::extract::State<leptos::LeptosOptions>,
        request: Request<Body>,
    ) -> axum::response::Response {
        let handler = leptos_axum::render_app_async_with_context(
            option.clone(),
            move || {
                provide_context(app_state.clone());
                provide_context(auth_session.clone());
                provide_context(app_state.pool.clone());
            },
            move || view! {  <App/> },
        );

        handler(request).await.into_response()
    }
*/
    // ---------- Leptos SSR handler ----------
    async fn leptos_handler(
        State(app_state): State<AppState>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        let nonce = req.extensions().get::<Nonce>().cloned().unwrap_or_else(Nonce::new);
        let leptos_options = app_state.leptos_options.clone();
        let leptos_options_clone = leptos_options.clone();
        let auth_parameters:AuthParameters = app_state.config.auth_parameters();
        let handler = leptos_axum::render_app_to_stream_with_context(
            move || {
                println!("provide_context auth_parameters, leptos_options ");
                provide_context(Auth::signal());
                Auth::init(auth_parameters.clone());
                provide_context(leptos_options.clone());
                provide_context(nonce.clone());
                println!("after provide_context in leptos_handler");
            },
            move||  { shell(leptos_options_clone.clone()) },
            //move||  { App }, // Do not do this - no meta, no header, no nonce in scripts
        );
        handler(req).await.into_response()
    }

    // ---------- Security Middleware (post-render) ----------
/*    async fn security_headers_middleware<B>(
        req: Request<Body>,
        next: Next,
    ) -> axum::response::Response {
        let mut response = next.run(req).await;

        let nonce = extract_nonce_from_csp(&response);

        let mut headers = HeaderMap::new();
        headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
        headers.insert("Permissions-Policy", "geolocation=(), microphone=(), camera=()".parse().unwrap());
        headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        headers.insert("X-Frame-Options", "DENY".parse().unwrap());

        if let Some(nonce) = nonce {
            if let Some(csp) = response.headers().get("content-security-policy") {
                let mut value = csp.to_str().unwrap().to_string();
                value.push_str(&format!(" script-src 'nonce-{nonce}';"));
                headers.insert("Content-Security-Policy", value.parse().unwrap());
            }
        }

        let response_headers = response.headers_mut();
        for (k, v) in headers {
            response_headers.insert(k.unwrap(), v);
        }

        response
    }
*/
    // ---------- Helper ----------

/*    fn extract_nonce_from_csp(response: &axum::response::Response<Body>) -> Option<String> {
        response
            .headers()
            .get("content-security-policy")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| {
                val.split_whitespace()
                    .find(|part| part.starts_with("'nonce-"))
                    .map(|p| p.trim_matches('\'').trim_start_matches("nonce-").to_string())
            })
    }
*/
    async fn me(Authenticated(claims): Authenticated) -> String {
        format!("hello {}", claims.name.unwrap_or(claims.sub))
    }

    async fn admin_stats(Authenticated(claims): Authenticated) -> Result<&'static str, (axum::http::StatusCode, &'static str)> {
        ensure_role(&claims, Role::Admin)?;
        Ok("Top secret stats")
    }
    /// Middleware, with CSP 3 && security headers
    /// && nonce in Request Extensions, for Leptos
    pub async fn security_headers(
        State(_app_state): State<AppState>,
        mut req: Request<Body>,
        next: Next,
    ) -> impl IntoResponse {
        // TODO move to AppState
        let app_env = std::env::var("APP_ENV").ok()
            .unwrap_or_else(|| "PROD".to_string());
        let is_prod = matches!(app_env.as_str(), "PROD" | "prod" | "Production" | "production");

        let nonce = Nonce::new();
        req.extensions_mut().insert(nonce.clone());
        let mut res = next.run(req).await;
        let nonce = nonce.to_string();
        const CF:&str = "https://cdnjs.cloudflare.com";

        // Content-Security-Policy
        let csp = if !is_prod {
            // DEV: relaxed to support HMR/hydration;
            // include nonce as we prepare to remove 'unsafe-inline' later
            format!(
                "default-src 'self'; \
                frame-ancestors 'none'; \
                script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' 'nonce-{}'; \
                style-src 'self' 'unsafe-inline' {CF} 'nonce-{}'; \
                img-src 'self' data: blob:; \
                font-src 'self' data: {CF}; \
                connect-src 'self' ws: wss:",
                nonce, nonce
            )
        } else {
            // PROD: no ws/wss and no 'unsafe-eval'; keep 'unsafe-inline' for styles,
            // include nonce for scripts/styles
            format!(
                "default-src 'self';\
                 frame-ancestors 'none'; \
                 script-src 'self' 'nonce-{}' 'wasm-unsafe-eval'; \
                 style-src 'self' {CF} 'nonce-{}'; \
                 img-src 'self' data: blob:; \
                 font-src 'self' data: {CF}; \
                 connect-src 'self'",
                nonce, nonce
            )
        };

        let headers = res.headers_mut();
        headers.insert("Content-Security-Policy", HeaderValue::from_str(&csp).unwrap());
        headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
        headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
        headers.insert("Referrer-Policy", HeaderValue::from_static("no-referrer"));
        headers.insert("Cross-Origin-Embedder-Policy", HeaderValue::from_static("require-corp"));
        headers.insert("Cross-Origin-Opener-Policy", HeaderValue::from_static("same-origin"));
        headers.insert("Cross-Origin-Resource-Policy", HeaderValue::from_static("same-origin"));

        headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
        headers.insert("Permissions-Policy", HeaderValue::from_static("camera=(), microphone=(), geolocation=()"));
        // HSTS only in PROD (assumes HTTPS termination in front)
        if is_prod {
            headers.insert("Strict-Transport-Security", HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"));
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

