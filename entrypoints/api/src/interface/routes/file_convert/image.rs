use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use omnius_opxs_base::AppError;
use omnius_opxs_file_convert::{
    FileConvertImageInputFileType, FileConvertImageOutputFileType, FileConvertImageRequestParam,
    FileConvertJobStatus, FileConvertJobType,
};

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
    path = "/api/v1/file-convert/image/upload",
    request_body = UploadInput,
    responses(
        (status = 200, body = UploadOutput)
    )
)]
pub async fn upload(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<UploadInput>,
) -> Result<Json<UploadOutput>, AppError> {
    let job_id = state.service.tsid_provider.lock().gen().to_string();
    let param = FileConvertImageRequestParam {
        file_stem: input.file_stem,
        in_type: input.in_type,
        out_type: input.out_type,
    };
    let upload_uri = state
        .service
        .image_convert_job_creator
        .create_job(&job_id, &FileConvertJobType::Image, &param)
        .await?;

    Ok(Json(UploadOutput { job_id, upload_uri }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UploadInput {
    pub file_stem: String,
    pub in_type: FileConvertImageInputFileType,
    pub out_type: FileConvertImageOutputFileType,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct UploadOutput {
    pub job_id: String,
    pub upload_uri: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/file-convert/image/status",
    request_body = StatusInput,
    responses(
        (status = 200, body = StatusOutput)
    )
)]
pub async fn status(
    State(state): State<AppState>,
    input: Query<StatusInput>,
) -> Result<Json<StatusOutput>, AppError> {
    let (status, download_uri) = state
        .service
        .image_convert_job_creator
        .get_download_url(&input.job_id, &input.file_name)
        .await?;

    Ok(Json(StatusOutput {
        status,
        download_uri,
    }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct StatusInput {
    pub job_id: String,
    pub file_name: String,
}

#[derive(Serialize, ToSchema, Validate)]
pub struct StatusOutput {
    pub status: FileConvertJobStatus,
    pub download_uri: Option<String>,
}
