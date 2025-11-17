use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use leptos::config::LeptosOptions;
use oauth2::{CsrfToken, PkceCodeVerifier};
use openidconnect::Nonce;
use crate::auth::Role;
use crate::ssr::ISPOidcClient;

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: Arc<LeptosOptions>,
    pub oidc_client: Arc<ISPOidcClient>, // with config: AppConfig
    pub sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    pub async_http_client: reqwest::Client,
}

impl AppState {
    /// Initializes and returns the application state.
    ///
    /// This function is asynchronous because it initializes the OIDC client.
    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Initialize HTTP Client
        let async_http_client = reqwest::Client::new();

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
        };

        Ok(state)
    }
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
    pub pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub subject: Option<String>,
    pub name: Option<String>,
    pub roles: HashSet<Role>,
}