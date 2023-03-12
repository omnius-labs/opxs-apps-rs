use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub enum AppError {
    #[allow(unused)]
    InvalidRequest,
    MissingCredential,
    UserAlreadyExists,
    UserNotFound,
    UnexpectedError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidRequest => (StatusCode::BAD_REQUEST, "Invalid request"),
            AppError::MissingCredential => (StatusCode::UNAUTHORIZED, "Missing credential"),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
