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
}
