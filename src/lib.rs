
pub mod app;
pub mod components;
pub mod auth;
pub mod config;
pub mod error;
pub mod rbac;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

/*
#[cfg(not(feature = "ssr"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate3() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    use leptos::*;
    use leptos_oidc::Auth;
    use leptos::context::provide_context;
    leptos::mount::hydrate_body(|| {
        let auth_signal = Auth::signal(); // scope implicit
        provide_context(auth_signal);
        view! { cx, <App/> }
    });
}


// This block compiles only for the client (wasm)
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use leptos::*;
    use leptos::prelude::*;
    use leptos_oidc::*;
    //use leptos::task::spawn_local;
    use crate::app::App;
    use leptos::context::provide_context;
    //use gloo_net::http::Request;
    leptos::mount::hydrate_body(|| {
        // 1️⃣ Create and provide AuthSignal
        let auth_signal = Auth::signal();
        provide_context(auth_signal);

        let init_auth = LocalResource::new(move || async move {
            // Fetch auth parameters from the API
            let params = match reqwest::Client::new()
                .get("/api/get_auth_parameters")
                .send()
                .await
            {
                Ok(response) => match response.json::<AuthParameters>().await {
                    Ok(params) => params,
                    Err(e) => {
                        leptos::logging::error!("Failed to parse auth parameters: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    leptos::logging::error!("Failed to fetch auth parameters: {}", e);
                    return;
                }
            };

            // Initialize Auth with the parameters
            Auth::init(params);
        });
        init_auth.read();
        // 4️⃣ Render your app
        view! { <App/> }
    });
}

 */