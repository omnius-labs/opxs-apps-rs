use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{Error, ErrorKind, Result};

#[async_trait]
pub trait GoogleOAuth2Provider {
    async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2TokenResult>;
    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo>;
}

pub struct GoogleOAuth2ProviderImpl;

#[async_trait]
impl GoogleOAuth2Provider for GoogleOAuth2ProviderImpl {
    async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2TokenResult> {
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
            .await?;

        if res.status() != StatusCode::OK {
            let message = res.text().await?;
            return Err(Error::new(ErrorKind::GcpError).message(format!("get oauth2 token error: {}", message)));
        }

        let oauth2_token = res.json::<OAuth2Token>().await?;
        Ok(OAuth2TokenResult {
            access_token: oauth2_token.access_token.clone(),
            id_token_claims: oauth2_token.id_token_claims()?,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
        let client = reqwest::Client::new();
        let res = client
            .get("https://www.googleapis.com/oauth2/v1/userinfo")
            .query(&[("access_token", access_token.to_string())])
            .send()
            .await?;

        if res.status() != StatusCode::OK {
            let message = res.text().await?;
            return Err(Error::new(ErrorKind::GcpError).message(format!("get user info error: {}", message)));
        }

        let user_info = res.json::<UserInfo>().await?;
        Ok(user_info)
    }
}

#[derive(Deserialize)]
struct OAuth2Token {
    pub access_token: String,
    pub id_token: String,
}

impl OAuth2Token {
    pub fn id_token_claims(&self) -> Result<IdTokenClaims> {
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
