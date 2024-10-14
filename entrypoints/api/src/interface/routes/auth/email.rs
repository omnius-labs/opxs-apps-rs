use axum::{extract::State, routing::post, Json, Router};
use hyper::StatusCode;
use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;
use validator::Validate;

use omnius_opxs_auth::shared::model::{AuthToken, User};
use omnius_opxs_base::AppError;

use crate::{interface::extractors::ValidatedJson, shared::state::AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/confirm", post(confirm))
        .route("/unregister", post(unregister))
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

    let email_confirm_url = Url::parse_with_params(
        format!("{}auth/register/email/confirm", state.conf.web.origin.as_str()).as_str(),
        &[("token", token)],
    )?
    .to_string();

    let job_id = state.service.tsid_provider.lock().gen().to_string();
    state
        .service
        .email_send_job_creator
        .create_email_confirm_job(
            &job_id,
            &input.name,
            &input.email,
            &state.conf.email.from_email_address,
            &email_confirm_url,
        )
        .await?;

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
pub async fn confirm(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<ConfirmInput>) -> Result<Json<AuthToken>, AppError> {
    let user_id = state.service.email_auth.confirm(&input.token).await?;
    let auth_token = state.service.token.create(&user_id).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct ConfirmInput {
    pub token: String,
}
#[utoipa::path(
    post,
    path = "/api/v1/auth/email/unregister",
    request_body = RegisterInput,
    responses(
        (status = 200)
    )
)]
pub async fn unregister(State(state): State<AppState>, user: User) -> Result<StatusCode, AppError> {
    state.service.email_auth.unregister(user.id.as_str()).await?;
    Ok(StatusCode::OK)
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
