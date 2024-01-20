use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::shared::error::AuthError;

#[async_trait]
pub trait GoogleOAuth2Provider {
    async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2TokenResult, AuthError>;
    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, AuthError>;
}

pub struct GoogleOAuth2ProviderImpl;

#[async_trait]
impl GoogleOAuth2Provider for GoogleOAuth2ProviderImpl {
    async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2TokenResult, AuthError> {
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
            .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        if res.status() != StatusCode::OK {
            let message = res.text().await.map_err(|e| AuthError::UnexpectedError(e.into()))?;
            return Err(AuthError::UnexpectedError(anyhow::anyhow!("google get token error: {}", message)));
        }

        let oauth2_token = res.json::<OAuth2Token>().await.map_err(|e| AuthError::UnexpectedError(e.into()))?;
        Ok(OAuth2TokenResult {
            access_token: oauth2_token.access_token.clone(),
            id_token_claims: oauth2_token.id_token_claims()?,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, AuthError> {
        let client = reqwest::Client::new();
        let res = client
            .get("https://www.googleapis.com/oauth2/v1/userinfo")
            .query(&[("access_token", access_token.to_string())])
            .send()
            .await
            .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        if res.status() != StatusCode::OK {
            let message = res.text().await.map_err(|e| AuthError::UnexpectedError(e.into()))?;
            return Err(AuthError::UnexpectedError(anyhow::anyhow!("google get user info error: {}", message)));
        }

        let user_info = res.json::<UserInfo>().await.map_err(|e| AuthError::UnexpectedError(e.into()))?;
        Ok(user_info)
    }
}

#[derive(Deserialize)]
struct OAuth2Token {
    pub access_token: String,
    pub id_token: String,
}

impl OAuth2Token {
    pub fn id_token_claims(&self) -> anyhow::Result<IdTokenClaims> {
        let jwt: Vec<&str> = self.id_token.split('.').collect();
        let payload = jwt[1];
        let payload = BASE64.decode(payload)?;
        let payload = String::from_utf8(payload)?;
        let payload: IdTokenClaims = serde_json::from_str(&payload)?;
        Ok(payload)
    }
}

#[derive(Debug, Clone)]
pub struct OAuth2TokenResult {
    pub access_token: String,
    pub id_token_claims: IdTokenClaims,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
