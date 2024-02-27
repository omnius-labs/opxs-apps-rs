use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use opxs_base::AppError;
use opxs_image_convert::{ImageConvertJobStatus, ImageFormat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{interface::extractors::ValidatedJson, shared::state::AppState};

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
        (status = 200, body = UploadOutput)
    )
)]
pub async fn upload(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<UploadInput>) -> Result<Json<UploadOutput>, AppError> {
    let job_id = state.service.tsid_provider.gen().to_string();
    let upload_uri = state
        .service
        .image_convert_job_creator
        .create_image_convert_job(&job_id, &input.source_filename, &input.target_image_format)
        .await?;

    Ok(Json(UploadOutput { job_id, upload_uri }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UploadInput {
    pub source_filename: String,
    pub target_image_format: ImageFormat,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct UploadOutput {
    pub job_id: String,
    pub upload_uri: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/convert/file/status",
    request_body = StatusInput,
    responses(
        (status = 200, body = StatusOutput)
    )
)]
pub async fn status(State(state): State<AppState>, input: Query<StatusInput>) -> Result<Json<StatusOutput>, AppError> {
    let (status, download_uri) = state.service.image_convert_job_creator.get_status(&input.job_id).await?;

    Ok(Json(StatusOutput { status, download_uri }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct StatusInput {
    pub job_id: String,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct StatusOutput {
    pub status: ImageConvertJobStatus,
    pub download_uri: Option<String>,
}
