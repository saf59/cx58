// components/media_proxy_script.rs
use leptos::prelude::*;

#[component]
pub fn MediaProxyScript() -> impl IntoView {
    view! {
        {move || {
            #[cfg(feature = "ssr")]
            {
                if let Some(client_config) = use_context::<crate::state::ClientConfig>() {
                    let nonce = use_context::<leptos::nonce::Nonce>();

                    view! {
                        <script nonce=nonce>
                            {format!("window.MEDIA_PROXY = '{}';", client_config.media_proxy)}
                        </script>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }
            #[cfg(not(feature = "ssr"))]
            {
                view! {}.into_any()
            }
        }}
    }
}