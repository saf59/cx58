use crate::{app::LogoutButton, components::lang::LanguageSwitcher};
use leptos::prelude::*;

#[component]
pub fn SideBar<Top, SideBody>(top: Top, side_body: SideBody, children: Children) -> impl IntoView
where
    Top: IntoView + 'static,
    SideBody: IntoView + 'static,
{
    let (is_collapsed, set_is_collapsed) = signal(false);
    let (is_pinned, set_is_pinned) = signal(false);

    Effect::new(move |_| {
        if stored_sidebar_pinned() {
            set_is_pinned.set(true);
        }
    });

    view! {
        <div class="sb-wrapper" class:sb-collapsed=is_collapsed class:sb-pinned=is_pinned>
            <div class="sb-hoverStrip" on:mouseenter=move |_| set_is_collapsed.set(false)></div>
            <div
                class="sb-sidebar"
                on:mouseleave=move |_| {
                    if !is_pinned.get() {
                        set_is_collapsed.set(true)
                    }
                }
            >
                <header class="sb-sideHeader">
                    <button
                        class="tooltip sb-pin"
                        on:click=move |_| {
                            set_is_pinned.set(true);
                            set_is_collapsed.set(false);
                            store_sidebar_pinned(true);
                        }
                        data-descr="Pin sidebar"
                    />
                    <div class="sb-side-top">{top.into_view()}</div>
                    <button
                        class="tooltip sb-unpin"
                        on:click=move |_| {
                            set_is_pinned.set(false);
                            store_sidebar_pinned(false);
                            set_is_collapsed.set(true)
                        }
                        data-descr="Hide sidebar"
                    />
                </header>
                <div class="sb-sidebar-body">{side_body.into_view()}</div>
            </div>
            <div class="sb-content">
                <button class="sb-fakepin" />
                {children()}
            </div>
        </div>
        <LogoutButton />
        <LanguageSwitcher />
    }
}

#[cfg(target_arch = "wasm32")]
fn stored_sidebar_pinned() -> bool {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("cx58-sidebar-pinned").ok().flatten())
        .as_deref()
        == Some("true")
}

#[cfg(not(target_arch = "wasm32"))]
fn stored_sidebar_pinned() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn store_sidebar_pinned(pinned: bool) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item("cx58-sidebar-pinned", if pinned { "true" } else { "false" });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn store_sidebar_pinned(_pinned: bool) {}
