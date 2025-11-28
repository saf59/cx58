use leptos::prelude::{signal, ClassAttribute, ElementChild, Get, OnAttribute, Set};
use leptos::{component, view, IntoView};
use crate::app::LogoutButton;

#[component]
pub fn SideBar<Top,SideBody,Content>(
    top: Top,
    side_body: SideBody,
    content: Content,
) -> impl IntoView
where
    Top: IntoView + 'static,
    SideBody: IntoView + 'static,
    Content: IntoView + 'static,
{
    let (is_collapsed, set_is_collapsed) = signal(true);
    let (is_pinned, set_is_pinned) = signal(false);
    view! {
        <div class="sb-wrapper" class:sb-collapsed=is_collapsed class:sb-pinned=is_pinned>
            <div
                class="sb-sidebar"
                on:mouseleave=move |_| {
                    if !is_pinned.get() {
                        set_is_collapsed.set(true)
                    }
                }
            >
                <header class="sb-sideHeader">
                    <button class="tooltip sb-pin" on:click=move |_| set_is_pinned.set(true)>
                        <span class="tooltiptext">Pin sidebar</span>
                    </button>
                    <div class="sb-side-top">{top.into_view()}</div>
                    <button
                        class="tooltip sb-unpin"
                        on:click=move |_| {
                            set_is_pinned.set(false);
                            set_is_collapsed.set(true)
                        }
                    >
                        <span class="tooltiptext">Hide sidebar</span>
                    </button>
                </header>
                <div class="sb-sidebar-body">{side_body.into_view()}</div>
            </div>
            <div class="sb-content">
                <button class="sb-fakepin" />
                {content.into_view()}
            </div>
        </div>
        <div class="sb-hoverStrip" on:mouseenter=move |_| set_is_collapsed.set(false)></div>
        <LogoutButton />
    }
}
