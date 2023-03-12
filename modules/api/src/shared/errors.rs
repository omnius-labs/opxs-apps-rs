use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Error, ToSchema)]
pub enum AppError {
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    TokioRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(transparent)]
    AxumTypedHeaderError(#[from] axum::extract::rejection::TypedHeaderRejection),
    #[error(transparent)]
    AxumExtensionError(#[from] axum::extract::rejection::ExtensionRejection),
    #[error(transparent)]
    AxumJsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error("user not found")]
    UserNotFound,
    // #[error("wrong credentials")]
    // WrongCredentials,
    #[error("password doesn't match")]
    WrongPassword,
    #[error("duplicate user email")]
    DuplicateUserEmail,
    #[error("duplicate user name")]
    DuplicateUserName,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::AxumJsonRejection(_) => StatusCode::BAD_REQUEST,
            AppError::DuplicateUserEmail => StatusCode::CONFLICT,
            AppError::DuplicateUserName => StatusCode::CONFLICT,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{:?}", self);
        }

        let payload = json!({"message": self.to_string()});
        (status, Json(payload)).into_response()
    }
}
