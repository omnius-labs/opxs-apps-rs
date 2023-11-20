use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Error, ToSchema)]
pub enum AuthError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("register error")]
    RegisterRejection(anyhow::Error),
    #[error("login error")]
    LoginRejection(anyhow::Error),
    #[error("access token expired")]
    AccessTokenExpired,
    #[error("refresh token not found")]
    RefreshTokenNotFound,
    #[error("user not found")]
    UserNotFound,
    #[error("password doesn't match")]
    WrongPassword,
    #[error("duplicate email")]
    DuplicateEmail,
    #[error("email verify token expired")]
    EmailVerifyTokenExpired,
}
