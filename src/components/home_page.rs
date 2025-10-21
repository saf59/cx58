#![allow(unused_imports)]
use crate::app::{get_auth_parameters, get_config};
use crate::config::AppConfig;
use leptos::ev::{click, on};
use leptos::html::{br, button};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_oidc::{
    Auth, AuthErrorContext, AuthLoaded, AuthLoading, Authenticated, LoginLink, LogoutLink,
};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    // let cfg = use_context::<AppConfig>().expect("AppConfig context missing");
    // log!("{:?}", cfg);
    //let auth = Auth::signal();  //init(cfg.auth_parameters());
    //provide_context(auth);
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome  to CX58 AI agent!"</h1>
        <p>"Secure authentication with Rauthy SSO"</p>
        //<a href={login_url} class="btn">"Login with Rauthy"</a>
        <button on:click=on_click>"Click Me: " {count}</button>
        <br/>
        <SomeAuth />
    }
}

#[component]
pub fn SomeAuth() -> impl IntoView {
    view! {
        <button class="sb-fakepin"></button>
        // Generate Sign In link
        <LoginLink class="optional-class-attributes">Sign in</LoginLink>
        // Generate Sign Out link
        <LogoutLink class="optional-class-attributes">Sign Out</LogoutLink>
        <AuthLoaded>"This will be rendered only when the auth library is not loading anymore"</AuthLoaded>
        <AuthLoading>"This will be rendered only when the auth library is still loading"</AuthLoading>
        <Authenticated>"This will only be rendered if the user is authenticated"</Authenticated>
        <AuthErrorContext>"This will only be rendered if there was an error during authentication"</AuthErrorContext>
        <br/>
        // A more complex example with optional fallbacks for the loading and unauthenticated state
        <Authenticated  unauthenticated=move ||
            view! { "This will only be rendered if the user is unauthenticated" }
        >
        "This will only be rendered if the user is authenticated"
        </Authenticated>
    }
}
