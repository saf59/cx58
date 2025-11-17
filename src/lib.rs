pub mod app;
pub mod server_fn;
pub mod config;
pub mod components;
#[cfg(feature = "ssr")]
pub mod ssr;
#[cfg(feature = "ssr")]
pub mod state;
pub mod auth;
#[cfg(feature = "ssr")]
pub mod auth_ssr;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
