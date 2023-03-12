use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::Value;

use crate::shared::{AppError, AppState};

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200)
    )
)]
#[allow(unused)]
pub async fn health(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let ret = state.service.health.check().await?;
    Ok(Json(ret))
}
