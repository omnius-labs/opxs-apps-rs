use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::common::AppError;

pub struct GoogleOAuth2Provider;

impl GoogleOAuth2Provider {
    pub async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2Token, AppError> {
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

    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, AppError> {
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
}

#[derive(Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub id_token: String,
}

impl OAuth2Token {
    pub fn id_token_payload(&self) -> anyhow::Result<IdTokenPayload> {
        let jwt: Vec<&str> = self.id_token.split('.').collect();
        let payload = jwt[1];
        let payload = BASE64.decode(payload)?;
        let payload = String::from_utf8(payload)?;
        let payload: IdTokenPayload = serde_json::from_str(&payload)?;
        Ok(payload)
    }
}

#[derive(Deserialize)]
pub struct IdTokenPayload {
    pub sub: String,
    pub nonce: String,
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
