use leptos::prelude::{ClassAttribute, ElementChild};
use leptos::{IntoView, component, view};
use leptos_fluent::move_tr;

#[component]
pub fn SideTop() -> impl IntoView {
    view! { <div class="cx58-color">{move_tr!("logo")}</div> }
}
