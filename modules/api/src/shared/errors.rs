use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
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
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    // WebSocket
    #[error("web socket handshake error")]
    WebSocketHandshakeError(anyhow::Error),

    // Service
    #[error("world mismatch")]
    WorldMismatchError,
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
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtError(_) => StatusCode::BAD_REQUEST,
            AppError::TokioRecvError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AxumTypedHeaderError(_) => StatusCode::BAD_REQUEST,
            AppError::AxumExtensionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AxumJsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,

            AppError::WorldMismatchError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RegisterRejection(_) => StatusCode::BAD_REQUEST,
            AppError::LoginRejection(_) => StatusCode::BAD_REQUEST,
            AppError::AccessTokenExpired => StatusCode::UNAUTHORIZED,
            AppError::RefreshTokenNotFound => StatusCode::UNAUTHORIZED,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::WrongPassword => StatusCode::BAD_REQUEST,
            AppError::DuplicateEmail => StatusCode::CONFLICT,
            AppError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{:?}", self);
        }

        let payload = json!({"message": self.to_string()});
        (status, Json(payload)).into_response()
    }
}
