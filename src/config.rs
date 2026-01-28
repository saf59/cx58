use axum_extra::extract::cookie;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    #[serde(skip_serializing)] // Never send to client
    pub oidc_client_secret: String,
    pub oidc_redirect_uri: String,
    pub oidc_post_logout_redirect_uri:String,
    pub oidc_scopes: String,
    pub cookie_config: CookieConfig,
    pub trust_data_list:String,
    pub trust_connect_list:String,
    pub media_proxy:String,
    pub chat_config: ChatConfig,
    pub is_prod:bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatConfig {
	pub agent_api_url: String,
	pub agent_api_key: Option<String>,
	pub agent_model: String,
	pub max_duration_sec: u64,
	pub max_chat_tokens: usize
}
impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            agent_api_url: "http://localhost:11434/api/generate".to_string(),
            agent_api_key: None,
            agent_model: "llava:latest".to_string(),
            max_duration_sec: 120,
            max_chat_tokens: 2000,
        }
    }
}
impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        // Determine environment (DEV/PROD)
        let app_env = env::var("APP_ENV").ok()
            .unwrap_or_else(|| "PROD".to_string());
        let is_prod = matches!(
            app_env.as_str(),
            "PROD" | "prod" | "Production" | "production"
        );

        // Base cookie config by env
        let mut cookie = if is_prod {CookieConfig::default()}
        else {CookieConfig{secure:false,..Default::default()}};

        if !is_prod {
            // In DEV, allow non-secure cookies for http://127.0.0.1
            cookie.secure = false;
            cookie.same_site = SameSiteConfig::Lax;
        }

        // Overrides via env
        if let Ok(v) = env::var("COOKIE_SECURE") {
            cookie.secure = v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = env::var("COOKIE_HTTP_ONLY") {
            cookie.http_only = v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = env::var("COOKIE_SAMESITE") {
            cookie.same_site = match v.as_str() {
                "strict" | "Strict" => SameSiteConfig::Strict,
                "none" | "None" => SameSiteConfig::None,
                _ => SameSiteConfig::Lax,
            }
        }
        if let Ok(v) = env::var("COOKIE_MAX_AGE_SECS")
            && let Ok(parsed) = v.parse::<i64>() {
                cookie.max_age_secs = parsed;
            }
        
        if let Ok(v) = env::var("COOKIE_PATH") {
            cookie.path = v;
        }

        let mut chat_config = ChatConfig::default();
		if let Ok(v) = env::var("AGENT_API_URL") {
            chat_config.agent_api_url = v;
        }
		if let Ok(v) = env::var("AGENT_API_KEY") {
            chat_config.agent_api_key = Some(v);
        }
		if let Ok(v) = env::var("AGENT_MODEL") {
            chat_config.agent_model = v;
        }
		if let Ok(v) = env::var("MAX_DURATION_SEC") 
			&& let Ok(parsed) = v.parse::<u64>() {
            	chat_config.max_duration_sec = parsed;
			}
		if let Ok(v) = env::var("MAX_CHAT_TOKENS") 
			&& let Ok(parsed) = v.parse::<usize>() {
            	chat_config.max_chat_tokens = parsed;
			}	


        Ok(Self {
            oidc_issuer_url: env::var("OIDC_ISSUER_URL").expect("OIDC_ISSUER_URL must be set"),
            oidc_client_id: env::var("OIDC_CLIENT_ID").expect("OIDC_CLIENT_ID must be set"),
            oidc_client_secret: env::var("OIDC_CLIENT_SECRET")
                .expect("OIDC_CLIENT_SECRET must be set"), // Server-only
            oidc_redirect_uri: env::var("OIDC_REDIRECT_URI")
                .expect("OIDC_REDIRECT_URI must be set"),
            oidc_post_logout_redirect_uri: env::var("OIDC_POST_LOGOUT_REDIRECT_URI")
                .expect("OIDC_POST_LOGOUT_REDIRECT_URI must be set"),
            oidc_scopes: env::var("OIDC_SCOPES")
                .unwrap_or_else(|_| "openid profile email groups".to_string()),
            cookie_config: cookie,
            trust_data_list:env::var("TRUST_DATA_LIST").unwrap_or_else(|_| "".to_string()),
            trust_connect_list:env::var("TRUST_CONNECT_LIST").unwrap_or_else(|_| "".to_string()),
            media_proxy:env::var("MEDIA_PROXY").unwrap_or_else(|_| "".to_string()),
			chat_config,
            is_prod
        })
    }

}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SameSiteConfig {
    Strict,
    Lax,
    None,
}

impl From<SameSiteConfig> for cookie::SameSite {
    fn from(config: SameSiteConfig) -> Self {
        match config {
            SameSiteConfig::Strict => cookie::SameSite::Strict,
            SameSiteConfig::Lax => cookie::SameSite::Lax,
            SameSiteConfig::None => cookie::SameSite::None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CookieConfig {
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSiteConfig,
    pub max_age_secs: i64,
    pub path: String,
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            secure: false,
            http_only: true,
            same_site: SameSiteConfig::Lax,
            max_age_secs: 3600, // 1 hour
            path: "/".to_string(),
        }
    }
}
