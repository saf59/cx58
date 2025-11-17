#![allow(unused_imports)]
use crate::config::AppConfig;
use leptos::ev::{click, on};
use leptos::html::{br, button};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use crate::app::LogoutButton;

#[component]
pub fn HomePage() -> impl IntoView {
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h3>"Welcome  to CX58 AI agent!"</h3>
        <button on:click=on_click>"Click Me: " {count}</button>
        <LogoutButton />
    }
}
