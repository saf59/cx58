use leptos::prelude::{ClassAttribute, ElementChild, OnAttribute, RwSignal, Write};
use leptos::{component, view, IntoView};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    view! {
        <h1>"Welcome  to CX58 AI agent!"</h1>
        <br />
        <button on:click=on_click>"Click Me: " {count}</button>
        <button class="sb-fakepin"></button>
    }
}
