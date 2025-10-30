use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("No refresh token")]
    NoRefreshToken,
    #[error("Rate is over")]
    RateLimited,
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Token exchange failed with status: {0}")]
    ExchangeFailed(StatusCode),
    #[error("Failed to parse token response: {0}")]
    Parse(String),
}
