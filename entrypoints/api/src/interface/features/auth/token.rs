use axum::{
    Json, Router,
    extract::State,
    routing::{delete, post},
};
use hyper::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use omnius_opxs_auth::model::{AuthToken, User};

use crate::{Result, interface::extractors::ValidatedJson, shared::state::AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/refresh", post(refresh_token))
        .route("/refresh", delete(delete_token))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/token/refresh",
    request_body = RefreshInput,
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn refresh_token(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<RefreshInput>) -> Result<Json<AuthToken>> {
    let auth_token = state.service.token.refresh(&input.refresh_token).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[utoipa::path(
    delete,
    path = "/api/v1/auth/token/refresh",
    responses(
        (status = 200)
    )
)]
pub async fn delete_token(State(state): State<AppState>, user: User) -> Result<StatusCode> {
    state.service.token.delete(user.id.as_str()).await?;
    Ok(StatusCode::OK)
}
