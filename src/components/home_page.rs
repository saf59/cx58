use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_oidc::{AuthErrorContext, AuthLoaded, AuthLoading, Authenticated, LoginLink, LogoutLink};
use crate::app::get_public_config;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    let config_resource = Resource::new(|| (), |_| async { get_public_config().await });
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome  to CX58 AI agent!"</h1>
        <p>"Secure authentication with Rauthy SSO"</p>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || {
                    config_resource.get().map(|result| {
                        match result {
                            Ok(config) => {
                                let login_url = format!(
                                    "{}/oidc/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
                                    config.oidc_issuer_url, config.oidc_client_id, config.redirect_uri, config.scopes
                                );
                                view! {
                                    <a href={login_url} class="btn">"Login with Rauthy"</a>
                                }.into_any()
                            },
                            Err(_) => view! {
                                <p>"Error loading configuration"</p>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        <button on:click=on_click>"Click Me: " {count}</button>
        <button class="sb-fakepin"></button>
                // Generate Sign In link
        <LoginLink class="optional-class-attributes">Sign in</LoginLink>

        // Generate Sign Out link
        <LogoutLink class="optional-class-attributes">Sign Out</LogoutLink>

        <AuthLoaded>"This will be rendered only when the auth library is not loading anymore"</AuthLoaded>

        <AuthLoading>"This will be rendered only when the auth library is still loading"</AuthLoading>

        <Authenticated>"This will only be rendered if the user is authenticated"</Authenticated>

        <AuthErrorContext>"This will only be rendered if there was an error during authentication"</AuthErrorContext>

        // A more complex example with optional fallbacks for the loading and unauthenticated state
        <Authenticated
            unauthenticated=move || view! { "This will only be rendered if the user is unauthenticated" }
            //loading=move || view! { "this will only be rendered if the library is still loading" }
            >
                "This will only be rendered if the user is authenticated"
        </Authenticated>

    }
}
