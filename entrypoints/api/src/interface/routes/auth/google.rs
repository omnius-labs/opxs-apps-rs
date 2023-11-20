use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use opxs_auth::shared::model::AuthToken;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::{error::AppError, state::AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/nonce", get(nonce))
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/google/nonce",
    responses(
        (status = 200, body = NonceOutput)
    )
)]
pub async fn nonce(jar: SignedCookieJar) -> Result<(SignedCookieJar, Json<NonceOutput>), AppError> {
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
) -> Result<(SignedCookieJar, Json<AuthToken>), AppError> {
    let cookie_nonce: Option<String> = jar.get("nonce").map(|cookie| cookie.value().to_owned());
    if cookie_nonce.is_none() {
        return Err(AppError::InvalidRequest(anyhow::anyhow!("Nonce not found")));
    }
    let cookie_nonce = cookie_nonce.unwrap();
    let jar = jar.remove(Cookie::named("nonce"));

    let user_id = state
        .service
        .google_auth
        .register(&input.code, &input.redirect_uri, &cookie_nonce)
        .await?;

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
pub async fn login(State(state): State<AppState>, jar: SignedCookieJar, Json(input): Json<LoginInput>) -> Result<Json<AuthToken>, AppError> {
    let nonce: Option<String> = jar.get("nonce").map(|cookie| cookie.value().to_owned());
    if nonce.is_none() {
        return Err(AppError::InvalidRequest(anyhow::anyhow!("Nonce not found")));
    }
    let nonce = nonce.unwrap();

    let user_id = state.service.google_auth.login(&input.code, &input.redirect_uri, &nonce).await?;

    let auth_token = state.service.token.create(&user_id).await?;

    Ok(Json(auth_token))
}

#[derive(Deserialize, ToSchema)]
pub struct LoginInput {
    pub redirect_uri: String,
    pub code: String,
}
