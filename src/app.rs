#![allow(unused_imports)]
pub use crate::components::home_page::HomePage;
use crate::components::side_body::SideBody;
use crate::components::side_top::SideTop;
use crate::components::sidebar::SideBar;
#[cfg(feature = "ssr")]
use axum::extract::{FromRef, State};
#[cfg(feature = "ssr")]
use axum::response::IntoResponse;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos::server_fn::middleware::{Layer, Service};
#[cfg(feature = "ssr")]
use leptos_axum::render_app_to_stream;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_oidc::{Auth, AuthParameters, AuthSignal, Challenge};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};
//use crate::components::app_config_provider::AppConfigProvider;
use crate::config::{AppConfig, PublicConfig};

#[cfg(feature = "ssr")]
use crate::auth::get_profile_claims;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <CspNonceHead />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

// Server function to read CSP nonce from response headers inserted by middleware
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
        Ok(nonce)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}

#[component]
fn CspNonceHead() -> impl IntoView {
    // Fetch nonce via server fn once per render
    let res = Resource::new(|| (), |_| async move { get_csp_nonce().await.ok().flatten() });
    view! {
        <Suspense fallback=|| view!{ <></> }>
            <Show when=move || res.get().is_some() fallback=|| view!{ <></> }>
                {move || {
                    let nonce = res.get().unwrap();
                    view! {
                        <meta name="csp-nonce" content=nonce.clone() />
                        //<script nonce=nonce>{"/* CSP nonce wired */"}</script>
                    }
                }}
            </Show>
        </Suspense>
    }
}


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth: AuthSignal = Auth::signal();
    provide_context(auth); // for different type - different context
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cx58-client.css" />
        <Stylesheet href="https://use.fontawesome.com/releases/v5.6.1/css/all.css" />
        <Title text="CX58 AI agent" />
        //<Link rel="icon" href="/favicon.ico" />
        //{link().rel("icon").href("/favicon.ico").into_view()}
        // content for this welcome page
        <Router>
            <main style="display: flex;height: 100%;width: 100%;">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route
                        path=StaticSegment("")
                        view=|| {
                            view! {
                                <SideBar top=SideTop() side_body=SideBody() content=HomePage() />
                            }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}

// Server function to get public config
#[server]
pub async fn get_public_config() -> Result<PublicConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::Extension;
        use leptos_axum::extract;

        let Extension(config): Extension<AppConfig> = extract().await?;
        Ok(config.public_config())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError("Not available on client".to_string()))
    }
}
