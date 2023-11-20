use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use opxs_auth::shared::model::AuthToken;
use opxs_email_send::EmailConfirmRequestParam;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    interface::extractors::ValidatedJson,
    shared::{error::AppError, state::AppState},
};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/confirm", get(confirm))
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
    let token = state.service.email_auth.register(&input.name, &input.email, &input.password).await?;

    let email_confirm_url = format!("{}/auth/register/email/confirm?token={}", state.conf.web.origin, token);

    let param = EmailConfirmRequestParam {
        email: input.email,
        user_name: input.name,
        email_confirm_url,
    };
    state.service.email_send_job_creator.create_email_confirm_job(&param).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterInput {
    #[validate(length(min = 2))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 4))]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/email/confirm",
    request_body = RegisterInput,
    responses(
        (status = 200)
    )
)]
pub async fn confirm(State(state): State<AppState>, input: Query<EmailVerificationInput>) -> Result<StatusCode, AppError> {
    state.service.email_auth.confirm(&input.token).await?;

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
async fn login(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<LoginInput>) -> Result<Json<AuthToken>, AppError> {
    let user_id = state.service.email_auth.login(&input.email, &input.password).await?;

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
