use leptos::prelude::{ClassAttribute, ElementChild};
use leptos::{component, view, IntoView};

#[component]
pub fn SideBody() -> impl IntoView {
    view! {
        <a href="#" class="active">
            <i class="fas fa-qrcode"></i>
            <span>Dashboard</span>
        </a>
        <a href="#">
            <i class="fas fa-link"></i>
            <span>Shortcuts</span>
        </a>
        <a href="#">
            <i class="fas fa-stream"></i>
            <span>Overview</span>
        </a>
        <a href="#">
            <i class="fas fa-calendar"></i>
            <span>Events</span>
        </a>
        <a href="#">
            <i class="far fa-question-circle"></i>
            <span>About</span>
        </a>
        <a href="#">
            <i class="fas fa-sliders-h"></i>
            <span>Services</span>
        </a>
        <a href="#">
            <i class="far fa-envelope"></i>
            <span>Contact</span>
        </a>
    }
}
