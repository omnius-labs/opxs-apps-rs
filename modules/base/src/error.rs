use std::fmt;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
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
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    AxumTypedHeaderError(#[from] axum_extra::typed_header::TypedHeaderRejection),
    #[error(transparent)]
    AxumExtensionError(#[from] axum::extract::rejection::ExtensionRejection),
    #[error(transparent)]
    AxumJsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("invalid request")]
    InvalidRequest(anyhow::Error),

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

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_code) = match self {
            AppError::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::JwtError(_) => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            AppError::TokioRecvError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::AxumError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::AxumTypedHeaderError(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            AppError::AxumExtensionError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::AxumJsonRejection(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            AppError::UrlParseError(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),

            AppError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, ErrorCode::BadRequest),

            AppError::RegisterRejection(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::LoginRejection(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
            AppError::AccessTokenExpired => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            AppError::RefreshTokenNotFound => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, ErrorCode::UserNotFound),
            AppError::WrongPassword => (StatusCode::NOT_FOUND, ErrorCode::UserNotFound),
            AppError::DuplicateEmail => (StatusCode::CONFLICT, ErrorCode::DuplicateEmail),
            AppError::EmailVerifyTokenExpired => {
                (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized)
            }

            AppError::UnexpectedError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
            ),
        };

        let payload = json!({"error_code": error_code.to_string()});
        (status_code, Json(payload)).into_response()
    }
}

#[derive(Debug, Serialize)]
enum ErrorCode {
    InternalServerError,
    BadRequest,
    Unauthorized,
    UserNotFound,
    DuplicateEmail,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::InternalServerError => write!(f, "InternalServerError"),
            ErrorCode::BadRequest => write!(f, "BadRequest"),
            ErrorCode::Unauthorized => write!(f, "Unauthorized"),
            ErrorCode::UserNotFound => write!(f, "UserNotFound"),
            ErrorCode::DuplicateEmail => write!(f, "DuplicateEmail"),
        }
    }
}
