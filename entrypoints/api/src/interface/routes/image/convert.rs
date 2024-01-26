use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    interface::extractors::ValidatedJson,
    shared::{error::AppError, state::AppState},
};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new().route("/upload", post(upload)).with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/image/convert/upload",
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
}

#[derive(Serialize, ToSchema, Validate)]
pub struct UploadOutput {
    pub refresh_token: String,
}
