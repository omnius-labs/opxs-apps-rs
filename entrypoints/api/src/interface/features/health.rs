use axum::{Json, extract::State};
use serde_json::Value;

use crate::{prelude::*, shared::state::AppState};

#[utoipa::path(
    get,
    tag = "health",
    operation_id = "health",
    path = "/api/health",
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    )
)]
#[allow(unused)]
pub async fn check(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    let ret = match state.service.health.check().await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };
    Ok(Json(ret))
}
