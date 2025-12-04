use crate::{app::LogoutButton, components::lang::LanguageSwitcher};
use leptos::prelude::*;

#[component]
pub fn SideBar<Top, SideBody>(top: Top, side_body: SideBody, children: Children) -> impl IntoView
where
    Top: IntoView + 'static,
    SideBody: IntoView + 'static,
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
                    <button class="tooltip sb-pin" on:click=move |_| set_is_pinned.set(true) data-descr="Pin sidebar"/>
                    <div class="sb-side-top">{top.into_view()}</div>
                    <button
                        class="tooltip sb-unpin"
                        on:click=move |_| {
                            set_is_pinned.set(false);
                            set_is_collapsed.set(true)
                        }
                        data-descr="Hide sidebar"/>
                </header>
                <div class="sb-sidebar-body">{side_body.into_view()}</div>
            </div>
            <div class="sb-content">
                <button class="sb-fakepin" />
                {children()}
            </div>
        </div>
        <div class="sb-hoverStrip" on:mouseenter=move |_| set_is_collapsed.set(false)></div>
        <LogoutButton />
        <LanguageSwitcher />
    }
}
