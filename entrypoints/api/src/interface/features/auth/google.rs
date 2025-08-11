use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

use omnius_opxs_auth::model::{AuthToken, User};

use crate::{prelude::*, shared::state::AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/nonce", get(nonce))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/unregister", post(unregister))
        .with_state(state)
}

#[utoipa::path(
    get,
    tag = "auth",
    operation_id = "authGoogleNonce",
    path = "/api/v1/auth/google/nonce",
    responses(
        (status = 200, body = NonceOutput)
    )
)]
pub async fn nonce(jar: SignedCookieJar) -> ApiResult<(SignedCookieJar, Json<NonceOutput>)> {
    let value = Uuid::new_v4().simple().to_string();
    let jar = jar.add(Cookie::new("nonce", value.clone()));
    let res = Json(NonceOutput { value });
    Ok((jar, res))
}

#[derive(Serialize, ToSchema)]
pub struct NonceOutput {
    pub value: String,
}

#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authGoogleRegister",
    path = "/api/v1/auth/google/register",
    responses(
        (status = 200, body = AuthToken),
        (status = 500, body = ApiErrorMessage)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(input): Json<RegisterInput>,
) -> ApiResult<(SignedCookieJar, Json<AuthToken>)> {
    let Some(nonce) = jar.get("nonce").map(|cookie| cookie.value().to_owned()) else {
        return Err(ApiErrorCode::InvalidRequest);
    };

    let jar = jar.remove(Cookie::build("nonce"));

    let user_id = match state.service.google_auth.register(&input.code, &input.redirect_uri, &nonce).await {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "google auth register failed");
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

    Ok((jar, Json(auth_token)))
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterInput {
    pub redirect_uri: String,
    pub code: String,
}

#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authGoogleLogin",
    path = "/api/v1/auth/google/login",
    responses(
        (status = 200, body = AuthToken),
        (status = 500, body = ApiErrorMessage)
    )
)]
pub async fn login(State(state): State<AppState>, jar: SignedCookieJar, Json(input): Json<LoginInput>) -> ApiResult<Json<AuthToken>> {
    let Some(nonce) = jar.get("nonce").map(|cookie| cookie.value().to_owned()) else {
        return Err(ApiErrorCode::InvalidRequest);
    };

    let user_id = match state.service.google_auth.login(&input.code, &input.redirect_uri, &nonce).await {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "google auth login failed");
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

#[derive(Deserialize, ToSchema)]
pub struct LoginInput {
    pub redirect_uri: String,
    pub code: String,
}

#[utoipa::path(
    post,
    tag = "auth",
    operation_id = "authGoogleUnregister",
    path = "/api/v1/auth/google/unregister",
    responses(
        (status = 200),
        (status = 500, body = ApiErrorMessage)
    ),
    security(
        ("bearer_token" = [])
    )
)]
pub async fn unregister(State(state): State<AppState>, user: User) -> ApiResult<StatusCode> {
    if let Err(e) = state.service.google_auth.unregister(user.id.as_str()).await {
        warn!(error = ?e);
        return Err(ApiErrorCode::InternalServerError);
    }
    Ok(StatusCode::OK)
}
