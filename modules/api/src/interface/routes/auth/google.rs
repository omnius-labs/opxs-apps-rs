use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    routing::get,
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, Key, SignedCookieJar};
use serde::Serialize;
use utoipa::ToSchema;

use crate::shared::{AppError, AppState};

#[allow(unused)]
pub fn gen_router(state: &Arc<AppState>) -> Router {
    Router::new()
        .route("/nonce", get(nonce))
        .with_state(state.clone())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/google/nonce",
    responses(
        (status = 200)
    )
)]
pub async fn nonce(
    jar: SignedCookieJar,
    State(state): State<Arc<AppState>>,
) -> Result<Json<NonceOutput>, AppError> {
    todo!("aa")
    // state
    //     .service
    //     .auth
    //     .register(&req.name, &req.email, &req.password)
    //     .await?;

    // Ok(StatusCode::OK)
}

#[derive(Serialize, ToSchema)]
pub struct NonceOutput {
    pub token_type: String,
    pub expires_in: i32,
    pub access_token: String,
    pub refresh_token: String,
}
