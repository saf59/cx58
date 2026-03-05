use leptos::prelude::*;

/*#[component]
pub fn MediaProxyScript() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(client_config) = use_context::<crate::ClientConfig>() {
            let nonce = use_context::<leptos::nonce::Nonce>()
                .map(|n| n.to_string())
                .unwrap_or_default();
            let content = format!("window.MEDIA_PROXY = '{}';", client_config.media_proxy);
            return view! {
                <Script nonce=nonce>
                    {content}
                </Script>
            }.into_any();
        }
    }
    view! { <Script /> }.into_any()
}
*/
/*#[component]
pub fn MediaProxyScript() -> impl IntoView {
    view! {
        {move || {
            #[cfg(feature = "ssr")]
            {
                if let Some(client_config) = use_context::<crate::ClientConfig>() {
                    let nonce = use_context::<leptos::nonce::Nonce>();

                    view! {
                        <script nonce=nonce>
                            {format!("window.MEDIA_PROXY = '{}';", client_config.media_proxy)}
                        </script>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }
            #[cfg(not(feature = "ssr"))] { ().into_any() }
        }}
    }
}
*/
#[component]
pub fn MediaProxyScript() -> impl IntoView {
    let client_config = use_context::<crate::ClientConfig>();
    let nonce = use_context::<leptos::nonce::Nonce>();

    let media_proxy = client_config
        .map(|c| c.media_proxy)
        .unwrap_or_default();

    view! {
        <script nonce=nonce>
            {format!("window.MEDIA_PROXY = {};", serde_json::to_string(&media_proxy).unwrap())}
        </script>
    }
}