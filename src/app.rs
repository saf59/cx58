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
use leptos_oidc::{Auth, AuthParameters, AuthSignal, Challenge};
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
#[cfg(feature = "ssr")]
use leptos::server_fn::middleware::{Layer, Service};
#[cfg(feature = "ssr")]
use leptos_axum::{render_app_to_stream, ResponseOptions};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    println!("Build html with nonce");
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

pub fn shell2(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    println!("shell()");
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                //<CspNonceHead />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App3 />
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
        println!("Прочитал x-csp-nonce:{:?}", nonce.clone());
        Ok(nonce)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(None)
    }
}
#[component]
fn CspNonceHead() -> impl IntoView {
    // Try to extract nonce from request context (inserted in middleware)
    let nonce = use_context::<String>().unwrap_or_default();
    println!("CNH Прочитал x-csp-nonce:{:?}", nonce.clone());
    view! {
        <meta name="csp-nonce" content=nonce.clone() />
        <link nonce=nonce.clone() id="leptos" rel="stylesheet" href="/pkg/cx58-client.css" />
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" crossorigin="anonymous"/>
        //<script nonce=nonce>{"/* CSP nonce wired */"}</script>
    }
}

#[component]
fn CspNonceHead3() -> impl IntoView {
    // Try to extract nonce from request context (inserted in middleware)
    // let nonce = use_context::<String>().unwrap_or_default();

    // Fetch nonce via server fn once per render
    let res = Resource::new(
        || (),
        |_| async move { get_csp_nonce().await.ok().flatten() },
    );
    view! {
        <Suspense fallback=|| view!{ <></> }>
            <Show when=move || res.get().is_some() fallback=|| view!{ <></> }>
                {move || {
                    let nonce = res.get().unwrap();
                    leptos::logging::log!("nonce:{}",nonce.clone().unwrap_or("none".to_string()));
                    view! {
                        <meta name="csp-nonce" content=nonce.clone() />
                        <link crossorigin=nonce.clone() id="leptos" rel="stylesheet" href="/pkg/cx58-client.css" />
                        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" crossorigin="anonymous"/>
                        //<script nonce=nonce>{"/* CSP nonce wired */"}</script>
                    }
                }}
            </Show>
        </Suspense>
    }
}

#[component]
pub fn App3() -> impl IntoView {
    // **Important**: this component must NOT render <head>, <html> or DOCTYPE.
    // it should produce only body content.
    view! {
        <main>
            <h1>"Hello Leptos"</h1>
            <p>"This minimal App will not trigger the leptos_meta panic."</p>
        </main>
    }
}
#[component]
pub fn App() -> impl IntoView {
    //let auth = use_context::<AuthSignal>().expect("AuthSignal not present in LoginLink");
    //println!("{:?}",auth);
    //let auth_parameters_resource = Resource::new(|| (), |_| async { get_auth_parameters().await });
    //println!("{:?}", auth_parameters_resource);
    //let auth_parameters = use_context::<AuthParameters>().expect("AuthParameters context missing");
    //println!("{:?}", auth_parameters);
    println!("Render App");
    view! {
        <Router>
            <main>
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

// Server function to get auth parameters
#[server]
pub async fn get_auth_parameters() -> Result<AuthParameters, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::Extension;
        use leptos_axum::extract;

        let Extension(config): Extension<AppConfig> = extract().await?;
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

        let Extension(config): Extension<AppConfig> = extract().await?;
        Ok(config)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "Not available on client".to_string(),
        ))
    }
}
