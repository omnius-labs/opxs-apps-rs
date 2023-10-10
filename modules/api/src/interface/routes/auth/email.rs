use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use opxs_shared::message::batch::email_send::EmailConfirmRequestParam;

use crate::{
    common::{AppError, AppState},
    domain::auth::model::AuthToken,
    interface::extractors::ValidatedJson,
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

    let email_confirm_url = format!("{}/api/v1/auth/email/confirm?token={}", state.conf.web.origin, token);

    let email_confirm_request = EmailConfirmRequestParam {
        email: input.email,
        user_name: input.name,
        email_confirm_url,
    };
    state
        .service
        .send_email_sqs_sender
        .send_message(&serde_json::to_string(&email_confirm_request).unwrap())
        .await?;

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
