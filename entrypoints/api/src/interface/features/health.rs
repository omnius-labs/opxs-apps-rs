use axum::{extract::State, Json};
use serde_json::Value;

use omnius_opxs_base::AppError;

use crate::shared::state::AppState;

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200)
    )
)]
#[allow(unused)]
pub async fn check(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let ret = state.service.health.check().await?;
    Ok(Json(ret))
}
