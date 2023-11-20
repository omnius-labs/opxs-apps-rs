use axum::{http::StatusCode, response::IntoResponse};
use opxs_auth::shared::error::AuthError;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Error, ToSchema)]
pub enum AppError {
    // Library
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    TokioRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    AxumTypedHeaderError(#[from] axum::extract::rejection::TypedHeaderRejection),
    #[error(transparent)]
    AxumExtensionError(#[from] axum::extract::rejection::ExtensionRejection),
    #[error(transparent)]
    AxumJsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("world mismatch")]
    WorldMismatch,
    #[error("invalid request")]
    InvalidRequest(anyhow::Error),

    #[error(transparent)]
    AuthError(#[from] AuthError),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtError(_) => StatusCode::BAD_REQUEST,
            AppError::TokioRecvError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AxumError(_) => StatusCode::BAD_REQUEST,
            AppError::AxumTypedHeaderError(_) => StatusCode::BAD_REQUEST,
            AppError::AxumExtensionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AxumJsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,

            AppError::WorldMismatch => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidRequest(_) => StatusCode::BAD_REQUEST,

            AppError::AuthError(AuthError::RegisterRejection(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(AuthError::LoginRejection(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(AuthError::AccessTokenExpired) => StatusCode::UNAUTHORIZED,
            AppError::AuthError(AuthError::RefreshTokenNotFound) => StatusCode::UNAUTHORIZED,
            AppError::AuthError(AuthError::UserNotFound) => StatusCode::NOT_FOUND,
            AppError::AuthError(AuthError::WrongPassword) => StatusCode::NOT_FOUND,
            AppError::AuthError(AuthError::DuplicateEmail) => StatusCode::CONFLICT,
            AppError::AuthError(AuthError::EmailVerifyTokenExpired) => StatusCode::UNAUTHORIZED,
            AppError::AuthError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            AppError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{:?}", self);
        }

        status.into_response()
    }
}
