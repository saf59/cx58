use crate::auth_ssr::SessionData;
use crate::ssr::ISPOidcClient;
use leptos::config::LeptosOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::extract::{FromRef, State};
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

impl FromRef<AppState> for State<AppState> {
    fn from_ref(state: &AppState) -> Self {
        // Мы просто клонируем Arc<AppState> и оборачиваем его в State
        // (Это работает, потому что AppState внутри State - это Arc)
        State(state.clone())
    }
}
impl FromRef<()> for AppState {
    fn from_ref(_state: &()) -> Self {
        // Мы не можем получить AppState из пустого контекста `&()`.
        // ВАЖНО: Мы не должны дойти до этой реализации.
        // Это заглушка, которая удовлетворяет компилятору,
        // но является индикатором неправильной настройки.
        //
        // ❌ Это решение, если бы вы извлекали State<T> из () в стандартном Axum хендлере.
        // Поскольку вы используете leptos_axum::extract, нам нужна другая стратегия.

        panic!("This FromRef<()> implementation should not be reached when using leptos_axum::extract().");
    }
}