use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    interface::extractors::ValidatedJson,
    shared::{AppError, AppState},
};

#[allow(unused)]
pub fn gen_router(state: &Arc<AppState>) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/verification", post(register))
        .route("/login", post(login))
        .with_state(state.clone())
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/register",
    request_body = RegisterInput,
    responses(
        (status = 200)
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<RegisterInput>,
) -> Result<StatusCode, AppError> {
    state
        .service
        .auth
        .register(&req.name, &req.email, &req.password)
        .await?;

    Ok(StatusCode::OK)
}

// #[utoipa::path(
//     post,
//     path = "/api/v1/auth/email-verification",
//     request_body = RegisterInput,
//     responses(
//         (status = 200)
//     )
// )]
// pub async fn email_verification(
//     State(state): State<Arc<AppState>>,
//     ValidatedJson(req): ValidatedJson<EmailVerificationInput>,
// ) -> Result<StatusCode, AppError> {
//     state
//         .service
//         .auth
//         .email_verification(&req.token)
//         .await?;

//     Ok(StatusCode::OK)
// }

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/login",
    request_body = LoginInput,
    responses(
        (status = 200, body = RegisterOutput)
    )
)]
async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginInput>,
) -> Result<Json<LoginOutput>, AppError> {
    let result = state.service.auth.login(&req.email, &req.password).await?;

    Ok(Json(LoginOutput {
        token_type: "bearer".to_string(),
        expires_in: result.expires_in,
        access_token: result.access_token,
        refresh_token: result.refresh_token,
    }))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterInput {
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct EmailVerificationInput {
    pub token: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginOutput {
    pub token_type: String,
    pub expires_in: i32,
    pub access_token: String,
    pub refresh_token: String,
}
