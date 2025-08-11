use axum::{Json, Router, extract::State, routing::post};
use hyper::StatusCode;
use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;
use validator::Validate;

use omnius_opxs_auth::model::{AuthToken, User};

use crate::{interface::extractors::ValidatedJson, prelude::*, shared::state::AppState};

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
    tag = "auth",
    operation_id = "authEmailRegister",
    path = "/api/v1/auth/email/register",
    request_body = RegisterInput,
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    )
)]
pub async fn register(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<RegisterInput>) -> ApiResult<StatusCode> {
    let token = match state.service.email_auth.register(&input.name, &input.email, &input.password).await {
        Ok(v) => v,
        Err(e) if *e.kind() == omnius_opxs_auth::ErrorKind::Duplicated => return Ok(StatusCode::OK),
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    let email_confirm_url = match Url::parse_with_params(
        format!("{}auth/register/email/confirm", state.conf.web.origin.as_str()).as_str(),
        &[("token", token)],
    ) {
        Ok(v) => v.to_string(),
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    let job_id = state.service.tsid_provider.lock().create().to_string();

    if let Err(e) = state
        .service
        .email_send_job_creator
        .create_job(
            &job_id,
            &input.name,
            &input.email,
            &state.conf.email.from_email_address,
            &email_confirm_url,
        )
        .await
    {
        warn!(error = ?e);
        return Err(ApiErrorCode::InternalServerError);
    }

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
    tag = "auth",
    operation_id = "authEmailConfirm",
    path = "/api/v1/auth/email/confirm",
    request_body = RegisterInput,
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    )
)]
pub async fn confirm(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<ConfirmInput>) -> ApiResult<Json<AuthToken>> {
    let user_id = match state.service.email_auth.confirm(&input.token).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    let auth_token = match state.service.token.create(&user_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct ConfirmInput {
    pub token: String,
}
#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authEmailUnregister",
    path = "/api/v1/auth/email/unregister",
    request_body = RegisterInput,
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    ),
    security(
        ("bearer_token" = [])
    )
)]
pub async fn unregister(State(state): State<AppState>, user: User) -> ApiResult<StatusCode> {
    if let Err(e) = state.service.email_auth.unregister(user.id.as_str()).await {
        warn!(error = ?e);
        return Err(ApiErrorCode::InternalServerError);
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authEmailLogin",
    path = "/api/v1/auth/email/login",
    request_body = LoginInput,
    responses(
        (status = 200, body = AuthToken),
        (status = 500, body = ApiErrorMessage)
    )
)]
async fn login(State(state): State<AppState>, ValidatedJson(input): ValidatedJson<LoginInput>) -> ApiResult<Json<AuthToken>> {
    let user_id = match state.service.email_auth.login(&input.email, &input.password).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::Unauthorized);
        }
    };

    let auth_token = match state.service.token.create(&user_id).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = ?e);
            return Err(ApiErrorCode::InternalServerError);
        }
    };

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
