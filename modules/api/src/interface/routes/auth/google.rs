use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use base64::engine::general_purpose::URL_SAFE_NO_PAD as BASE64;
use base64::engine::Engine as _;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::shared::{AppError, AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/nonce", get(nonce))
        .route("/register", post(register))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/google/nonce",
    responses(
        (status = 200)
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
        (status = 200)
    )
)]
pub async fn register(State(state): State<AppState>, jar: SignedCookieJar, Json(input): Json<RegisterInput>) -> Result<StatusCode, AppError> {
    let cookie_nonce: Option<String> = jar.get("nonce").map(|cookie| cookie.value().to_owned());
    if cookie_nonce.is_none() {
        return Err(AppError::RegisterError(anyhow::anyhow!("Nonce not found")));
    }
    let cookie_nonce = cookie_nonce.unwrap();

    let oauth2_token = get_oauth2_token(
        &input.code,
        &input.redirect_uri,
        &state.conf.auth.google.client_id,
        &state.conf.auth.google.client_secret,
    )
    .await?;

    let id_token_nonce = oauth2_token.nonce().map_err(AppError::UnexpectedError)?;
    if cookie_nonce != id_token_nonce {
        return Err(AppError::RegisterError(anyhow::anyhow!("Nonce mismatch error")));
    }

    let user_info = get_user_info(&oauth2_token.access_token).await?;
    info!("email: {}, name: {}", user_info.email, user_info.name);

    Ok(StatusCode::OK)
}

async fn get_oauth2_token(code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2Token, AppError> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://accounts.google.com/o/oauth2/token")
        .json(&json!({
            "client_id": client_id.to_string(),
            "client_secret": client_secret.to_string(),
            "grant_type": "authorization_code".to_string(),
            "redirect_uri": redirect_uri.to_string(),
            "code": code.to_string()
        }))
        .send()
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

    if res.status() != StatusCode::OK {
        let message = res.text().await.map_err(|e| AppError::UnexpectedError(e.into()))?;
        return Err(AppError::UnexpectedError(anyhow::anyhow!("google get token error: {}", message)));
    }

    let oauth2_token = res.json::<OAuth2Token>().await.map_err(|e| AppError::UnexpectedError(e.into()))?;
    Ok(oauth2_token)
}

async fn get_user_info(access_token: &str) -> Result<UserInfo, AppError> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .query(&[("access_token", access_token.to_string())])
        .send()
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

    if res.status() != StatusCode::OK {
        let message = res.text().await.map_err(|e| AppError::UnexpectedError(e.into()))?;
        return Err(AppError::UnexpectedError(anyhow::anyhow!("google get user info error: {}", message)));
    }

    let user_info = res.json::<UserInfo>().await.map_err(|e| AppError::UnexpectedError(e.into()))?;
    Ok(user_info)
}

#[derive(Deserialize)]
struct OAuth2Token {
    pub access_token: String,
    pub id_token: String,
}

impl OAuth2Token {
    pub fn nonce(&self) -> anyhow::Result<String> {
        let payload = self.id_token_payload()?;
        Ok(payload.nonce)
    }

    fn id_token_payload(&self) -> anyhow::Result<IdTokenPayload> {
        let jwt: Vec<&str> = self.id_token.split('.').collect();
        let payload = jwt[1];
        let payload = BASE64.decode(payload)?;
        let payload = String::from_utf8(payload)?;
        let payload: IdTokenPayload = serde_json::from_str(&payload)?;
        Ok(payload)
    }
}

#[derive(Deserialize)]
struct IdTokenPayload {
    pub nonce: String,
}

#[derive(Deserialize)]
struct UserInfo {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterInput {
    pub redirect_uri: String,
    pub code: String,
}
