pub mod app;
pub mod auth;
#[cfg(feature = "ssr")]
pub mod auth_ssr;
#[cfg(feature = "ssr")]
pub mod chunk_assembler;
pub mod components;
pub mod config;
#[cfg(feature = "ssr")]
pub mod llm_stream;
pub mod server_fn;
#[cfg(feature = "ssr")]
pub mod ssr;
#[cfg(feature = "ssr")]
pub mod state;
#[cfg(feature = "ssr")]
pub mod stop;
#[cfg(feature = "ssr")]
pub mod proxy_tree;

pub mod events;
#[cfg(feature = "ssr")]
pub mod hmac;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
