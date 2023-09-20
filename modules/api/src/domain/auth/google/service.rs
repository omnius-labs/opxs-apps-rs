use std::sync::Arc;

use crate::shared::{AppError, AuthConfig};

use super::{GoogleOAuth2Provider, ProviderAuthRepo};

#[derive(Clone)]
pub struct GoogleAuthService {
    pub oauth2_provider: Arc<dyn GoogleOAuth2Provider + Send + Sync>,
    pub auth_repo: Arc<dyn ProviderAuthRepo + Send + Sync>,
    pub auth_conf: AuthConfig,
}

impl GoogleAuthService {
    pub async fn register(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<i64, AppError> {
        let oauth2_token = self
            .oauth2_provider
            .get_oauth2_token(
                auth_code,
                auth_redirect_uri,
                &self.auth_conf.google.client_id,
                &self.auth_conf.google.client_secret,
            )
            .await?;

        let id_token_payload = oauth2_token.id_token_payload()?;

        if auth_nonce != id_token_payload.nonce {
            return Err(AppError::RegisterRejection(anyhow::anyhow!("Nonce mismatch error")));
        }

        if let Ok(user) = self.auth_repo.get_user("google", &id_token_payload.sub).await {
            return Ok(user.id);
        }

        let user_info = self.oauth2_provider.get_user_info(&oauth2_token.access_token).await?;

        let user_id = self.auth_repo.create_user(&user_info.name, "google", &id_token_payload.sub).await?;

        Ok(user_id)
    }

    pub async fn login(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<i64, AppError> {
        let oauth2_token = self
            .oauth2_provider
            .get_oauth2_token(
                auth_code,
                auth_redirect_uri,
                &self.auth_conf.google.client_id,
                &self.auth_conf.google.client_secret,
            )
            .await?;

        let id_token_payload = oauth2_token.id_token_payload()?;

        if auth_nonce != id_token_payload.nonce {
            return Err(AppError::LoginRejection(anyhow::anyhow!("Nonce mismatch error")));
        }

        let user = self.auth_repo.get_user("google", &id_token_payload.sub).await?;

        Ok(user.id)
    }

    // pub async fn unregister(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }
}
