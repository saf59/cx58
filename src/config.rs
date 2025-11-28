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
    pub agent_url:String,
    pub default_model:String,
    pub is_prod:bool
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
            agent_url:env::var("AGENT_API_URL")
            .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string()),
            default_model:env::var("AGENT_MODEL").unwrap_or_else(|_| "llava:latest".to_string()),
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
