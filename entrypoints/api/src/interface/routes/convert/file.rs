use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use opxs_convert_file::message::{ConvertFileFormat, ConvertFileStatus};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    interface::extractors::ValidatedJson,
    shared::{error::AppError, state::AppState},
};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/upload", post(upload))
        .route("/status", get(status))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/convert/file/upload",
    request_body = UploadInput,
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn upload(State(_state): State<AppState>, ValidatedJson(_input): ValidatedJson<UploadInput>) -> Result<Json<UploadOutput>, AppError> {
    todo!();
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UploadInput {
    pub filename: String,
    pub destination_file_format: ConvertFileFormat,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct UploadOutput {
    pub convert_id: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/convert/file/status",
    request_body = UploadInput,
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn status(State(_state): State<AppState>, _input: Query<StatusInput>) -> Result<Json<StatusOutput>, AppError> {
    todo!();
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct StatusInput {
    pub convert_id: String,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct StatusOutput {
    pub status: ConvertFileStatus,
    pub download_url: Option<String>,
}
