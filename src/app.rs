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

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth: AuthSignal = Auth::signal();
    provide_context(auth); // for different type - different context
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cx58.css" />
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

/*fn provide_all(resource: Resource<Result<PublicConfig, ServerFnError>>) {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    //let profile = Resource::new(|| (), |_| async { get_profile_claims().await });
    let public_config: PublicConfig = resource.get().expect("111").unwrap();

    let auth_parameters = AuthParameters {
        issuer: public_config.oidc_issuer_url.clone(),
        client_id: public_config.oidc_client_id.clone(),
        redirect_uri: public_config.redirect_uri.clone(),
        post_logout_redirect_uri: format!("{}/logout", public_config.redirect_uri.clone()),
        challenge: Challenge::S256,
        scope: Some("openid email profile".to_string()),
        audience: None,
    };
    provide_context(public_config);

    let _auth = Auth::init(auth_parameters);
}
*/
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
