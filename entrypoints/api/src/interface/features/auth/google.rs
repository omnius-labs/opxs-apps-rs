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

use omnius_core_base::hook_err;

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
    path = "/api/v1/auth/google/nonce",
    responses(
        (status = 200, body = NonceOutput)
    )
)]
pub async fn nonce(jar: SignedCookieJar) -> Result<(SignedCookieJar, Json<NonceOutput>)> {
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
    path = "/api/v1/auth/google/register",
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(input): Json<RegisterInput>,
) -> Result<(SignedCookieJar, Json<AuthToken>)> {
    let nonce = jar
        .get("nonce")
        .map(|cookie| cookie.value().to_owned())
        .ok_or_else(|| Error::new(ErrorKind::InvalidRequest).message("nonce not found"))?;
    let jar = jar.remove(Cookie::build("nonce"));

    let user_id = hook_err!(
        state.service.google_auth.register(&input.code, &input.redirect_uri, &nonce).await,
        |e| error!(error = %e, "google auth register failed")
    )?;

    let auth_token = state.service.token.create(&user_id).await?;

    Ok((jar, Json(auth_token)))
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterInput {
    pub redirect_uri: String,
    pub code: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/google/login",
    responses(
        (status = 200, body = AuthToken)
    )
)]
pub async fn login(State(state): State<AppState>, jar: SignedCookieJar, Json(input): Json<LoginInput>) -> Result<Json<AuthToken>> {
    let nonce = jar
        .get("nonce")
        .map(|cookie| cookie.value().to_owned())
        .ok_or_else(|| Error::new(ErrorKind::InvalidRequest).message("nonce not found"))?;

    let user_id = hook_err!(
        state.service.google_auth.login(&input.code, &input.redirect_uri, &nonce).await,
        |e| error!(error = %e, "google auth login failed")
    )?;

    let auth_token = state.service.token.create(&user_id).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema)]
pub struct LoginInput {
    pub redirect_uri: String,
    pub code: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/google/unregister",
    responses(
        (status = 200)
    )
)]
pub async fn unregister(State(state): State<AppState>, user: User) -> Result<StatusCode> {
    state.service.google_auth.unregister(user.id.as_str()).await?;
    Ok(StatusCode::OK)
}
