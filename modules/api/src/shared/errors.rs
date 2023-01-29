use axum::{response::IntoResponse, Json};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub enum AppError {
    InvalidRequest,
    MissingCredential,
    UserAlreadyExists,
    UserNotFound,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidRequest => (axum::http::StatusCode::BAD_REQUEST, "Invalid request"),
            AppError::MissingCredential => {
                (axum::http::StatusCode::UNAUTHORIZED, "Missing credential")
            }
            _ => (axum::http::StatusCode::BAD_REQUEST, "Invalid request"),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
