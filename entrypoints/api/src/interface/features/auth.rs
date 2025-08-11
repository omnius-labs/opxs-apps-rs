pub mod email;
pub mod google;

use axum::{
    Json, Router,
    extract::State,
    routing::{delete, get, post},
};
use hyper::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use omnius_opxs_auth::model::{AuthToken, User};

use crate::{interface::extractors::ValidatedJson, prelude::*, shared::state::AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/token", post(token_refresh))
        .route("/token", delete(token_delete))
        .nest_service("/email", email::gen_service(state.clone()))
        .nest_service("/google", google::gen_service(state.clone()))
        .with_state(state)
}

#[utoipa::path(
    get,
    tag = "auth",
    operation_id = "authMe",
    path = "/api/v1/auth/me",
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    ),
    security(
        ("bearer_token" = [])
    )
)]
pub async fn me(user: User) -> ApiResult<Json<User>> {
    Ok(Json(user))
}

#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authTokenRefresh",
    path = "/api/v1/token",
    request_body = RefreshInput,
    responses(
        (status = 200, body = AuthToken),
        (status = 500, body = ApiErrorMessage)
    )
)]
pub async fn token_refresh(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<RefreshInput>) -> ApiResult<Json<AuthToken>> {
    let auth_token = match state.service.token.refresh(&input.refresh_token).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RefreshInput {
    pub refresh_token: String,
}

#[utoipa::path(
    delete,
    tag = "auth",
    operation_id = "authTokenDelete",
    path = "/api/v1/auth/token",
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    ),
    security(
        ("bearer_token" = [])
    )
)]
pub async fn token_delete(State(state): State<AppState>, user: User) -> ApiResult<StatusCode> {
    if let Err(e) = state.service.token.delete(user.id.as_str()).await {
        warn!(error = ?e);
        return Err(ApiErrorCode::InternalServerError);
    }

    Ok(StatusCode::OK)
}
