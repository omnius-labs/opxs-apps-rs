use axum::{extract::State, routing::post, Json, Router};
use opxs_auth::shared::model::AuthToken;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    interface::extractors::ValidatedJson,
    shared::{error::AppError, state::AppState},
};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new().route("/refresh", post(refresh)).with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/token/refresh",
    request_body = RefreshInput,
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn refresh(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<RefreshInput>) -> Result<Json<AuthToken>, AppError> {
    let auth_token = state.service.token.refresh(&input.refresh_token).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshInput {
    pub refresh_token: String,
}
