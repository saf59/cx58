use leptos::prelude::{ClassAttribute, ElementChild};
use leptos::{component, view, IntoView};

#[component]
pub fn SideBody() -> impl IntoView {
    view! {
        <a href="#">
            <i class="fas fa-edit"></i>
            <span>New chat</span>
        </a>
        <a href="#">
            <i class="fas fa-book"></i>
            <span>FAQ</span>
        </a>
        <a href="#">
            <i class="fas fa-building"></i>
            <span>Objects</span>
        </a>
        <a href="#">
            <i class="fas fa-users"></i>
            <span>Users</span>
        </a>
    }
}
