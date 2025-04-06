use axum::{Json, extract::State};
use serde_json::Value;

use crate::{Result, shared::state::AppState};

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200)
    )
)]
#[allow(unused)]
pub async fn check(State(state): State<AppState>) -> Result<Json<Value>> {
    let ret = state.service.health.check().await?;
    Ok(Json(ret))
}
