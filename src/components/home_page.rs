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

#[component]
pub fn HomePage() -> impl IntoView {
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h3>"Welcome  to CX58 AI agent!"</h3>
        <button on:click=on_click>"Click Me: " {count}</button>
        <LogoutLink class="sign_out"><i class="fa fa-sign-out"></i><span>Sign out</span></LogoutLink>
    }
}
