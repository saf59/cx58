#![allow(unused_imports)]

#[cfg(feature = "ssr")]
use crate::auth::AppState;
pub use crate::components::home_page::HomePage;
use crate::components::side_body::SideBody;
use crate::components::side_top::SideTop;
use crate::components::sidebar::SideBar;
use crate::config::AppConfig;
use base64::Engine;
use leptos::{
    attr::{crossorigin, Scope},
    html::Link,
    prelude::*,
};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_oidc::{
    Auth, AuthLoaded, AuthParameters, AuthSignal, Authenticated, Challenge, LoginLink, LogoutLink,
};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::auth::get_profile_claims;
#[cfg(feature = "ssr")]
use axum::{
    extract::{FromRef, Request, State},
    http::HeaderValue,
    response::{Html, IntoResponse, Response},
};
use leptos::__reexports::wasm_bindgen_futures::JsFuture;
use leptos::reactive::spawn_local;
#[cfg(feature = "ssr")]
use leptos::server_fn::middleware::{Layer, Service};
#[cfg(feature = "ssr")]
use leptos_axum::{render_app_to_stream, ResponseOptions};
use web_sys::RequestInit;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <Title text="CX58 AI agent" />
                <AutoReload options=options.clone() />
                <Stylesheet id="leptos" href="/pkg/cx58-client.css" />
                <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    use leptos::prelude::*;
    use leptos_oidc::{Auth, AuthParameters};

    let auth_signal = Auth::signal();
    provide_context(auth_signal);

    // Move everything into a child component that lives *inside* Router
    view! {
        <Router>
            <AuthInitializer/>
        </Router>
    }
}

#[component]
fn AuthInitializer() -> impl IntoView {
    use leptos_oidc::{Auth, AuthParameters};

    // Fetch and initialize
    let auth_initialized: LocalResource<Result<(), String>> =
        LocalResource::new(move || async move {
            leptos::logging::log!("Fetching auth parameters...");
            let params: AuthParameters = gloo_net::http::Request::get("/api/get_auth_parameters")
                .send()
                .await
                .map_err(|e| format!("Fetch failed: {:?}", e))?
                .json()
                .await
                .map_err(|e| format!("Parse failed: {:?}", e))?;

            leptos::logging::log!("Initializing Auth...");
            leptos::logging::log!("{:?}", &params);
            Auth::init(params);
            Ok::<(), String>(())
        });

    view! {
        <Suspense fallback=|| view! { <div class="sb-content">"Initializing authentication..."</div> }>
            {move || {
                auth_initialized.get().map(|_| {
                    view! {
                        <Transition  >
                            <AuthLoaded>
                                <Authenticated unauthenticated=Unauthenticated>
                                    <SideBar top=SideTop() side_body=SideBody() content=HomePage() />
                                </Authenticated>
                            </AuthLoaded>

                    </Transition>
                        }
                })
            }}
        </Suspense>
    }
}

#[component]
pub fn Unauthenticated() -> impl IntoView {
    // <Title text="Unauthenticated"/>
    view! {
        <div class="sb-content">
            <h3>"Welcome  to CX58!"</h3>
            <h3>You are unauthenticated!</h3>
            <span>Please
            <LoginLink class="sign-in"><i class="fa fa-sign-in"></i><span>Sign in</span></LoginLink>
            via SSO.</span>
            <LoginButton/>
        </div>
        // Your Unauthenticated Page
    }
}
#[component]
pub fn LoginButton() -> impl IntoView {
    let on_click = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/api/start_login");
        }
    };

    view! {
        <button on:click=on_click class="btn btn-primary">
            "Login"
        </button>
    }
}
#[component]
pub fn LogoutButton() -> impl IntoView {
    let on_click = move |_| {
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/api/logout");
        }
    };

    view! {
        <button on:click=on_click class="btn btn-secondary">
            "Logout"
        </button>
    }
}


#[server]
pub async fn get_csp_nonce() -> Result<Option<String>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;

        let headers: HeaderMap = extract().await?;
        let nonce = headers
            .get("x-csp-nonce")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        println!("Read x-csp-nonce:{:?}", nonce.clone());
        Ok(nonce)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}
// Server function to get auth parameters
#[server]
pub async fn get_auth_parameters() -> Result<AuthParameters, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::Extension;
        use leptos_axum::extract;
        use std::sync::Arc;

        let Extension(config): Extension<Arc<AppConfig>> = extract().await?;
        Ok(config.auth_parameters())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Not available on client".to_string(),
        ))
    }
}
#[server]
pub async fn get_config() -> Result<AppConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::Extension;
        use leptos_axum::extract;
        use std::sync::Arc;

        let Extension(config): Extension<Arc<AppConfig>> = extract().await?;
        Ok(config.as_ref().clone())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Not available on client".to_string(),
        ))
    }
}
