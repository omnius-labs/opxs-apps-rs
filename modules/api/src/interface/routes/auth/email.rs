use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    domain::auth::model::AuthToken,
    interface::extractors::ValidatedJson,
    shared::{AppError, AppState},
};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/verify", post(verify))
        .route("/login", post(login))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/register",
    request_body = RegisterInput,
    responses(
        (status = 200)
    )
)]
pub async fn register(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<RegisterInput>) -> Result<StatusCode, AppError> {
    let _ = state.service.email_auth.register(&input.name, &input.email, &input.password).await?;

    Ok(StatusCode::OK)
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

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/verify",
    request_body = RegisterInput,
    responses(
        (status = 200)
    )
)]
pub async fn verify(State(state): State<AppState>, ValidatedJson(req): ValidatedJson<EmailVerificationInput>) -> Result<StatusCode, AppError> {
    state.service.email_auth.verify(&req.token).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct EmailVerificationInput {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/login",
    request_body = LoginInput,
    responses(
        (status = 200, body = AuthToken)
    )
)]
async fn login(State(state): State<AppState>, ValidatedJson(req): ValidatedJson<LoginInput>) -> Result<Json<AuthToken>, AppError> {
    let user_id = state.service.email_auth.login(&req.email, &req.password).await?;

    let auth_token = state.service.token.create(&user_id).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
