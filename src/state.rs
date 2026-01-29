use crate::auth_ssr::SessionData;
use crate::ssr::ISPOidcClient;
use leptos::config::LeptosOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: Arc<LeptosOptions>,
    pub oidc_client: Arc<ISPOidcClient>, // with config: AppConfig
    pub sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    pub async_http_client: reqwest::Client,
    pub chat_sessions: Arc<Mutex<HashMap<String, Arc<ChatSession>>>>
}
pub struct ChatSession {
    pub current_request_id: tokio::sync::RwLock<Option<String>>,
}
#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub media_proxy: String,
}

impl AppState {
    /// Initializes and returns the application state.
    ///
    /// This function is asynchronous because it initializes the OIDC client.
    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Initialize HTTP Client
        //let async_http_client = reqwest::Client::new();
        let async_http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build().expect("async_http_client build");

        // 2. Load Configuration and Options
        let conf = leptos::prelude::get_configuration(None)?;
        let leptos_options = conf.leptos_options;

        // 3. Initialize OIDC Client
        let oidc_client = ISPOidcClient::new(&async_http_client).await?;

        // 4. Construct AppState
        let state = AppState {
            // Options are cloned before being put into Arc,
            // as they are typically used by the server setup too.
            leptos_options: Arc::new(leptos_options),
            // The OIDC client is put into an Arc.
            oidc_client: Arc::new(oidc_client),
            // The sessions map is initialized empty and wrapped in Arc<Mutex<...>>.
            sessions: Arc::new(Mutex::new(HashMap::new())),
            // The reqwest::Client is typically cheap to clone for use in AppState.
            async_http_client: async_http_client.clone(),

            chat_sessions: Arc::new(Mutex::new(HashMap::new())),
        };

        Ok(state)
    }
}
