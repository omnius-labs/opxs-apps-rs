use std::sync::Arc;

use chrono::{Duration, Utc};

use core_base::{clock::SystemClock, random_bytes::RandomBytesProvider};

use crate::shared::{config::JwtConfig, error::AuthError, jwt, model::AuthToken};

use super::TokenRepo;

pub struct TokenService {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub jwt_conf: JwtConfig,
    pub token_repo: Arc<TokenRepo>,
}

impl TokenService {
    pub async fn create(&self, user_id: &i64) -> Result<AuthToken, AuthError> {
        let now = self.system_clock.now();

        let sub = user_id.to_string();
        let expires_in = Duration::days(14);
        let access_token = jwt::sign(&self.jwt_conf.secret.current, &sub, expires_in, now)?;
        let refresh_token = hex::encode(self.random_bytes_provider.get_bytes(32));
        let expires_at = now + expires_in;

        self.token_repo.create_token(user_id, &refresh_token, &expires_at).await?;

        Ok(AuthToken {
            expires_in: expires_in.num_seconds() as i32,
            access_token,
            refresh_token,
        })
    }

    pub async fn delete(&self, refresh_token: &str) -> Result<(), AuthError> {
        self.token_repo.delete_token(refresh_token).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthToken, AuthError> {
        let now = self.system_clock.now();
        let user_id = self.token_repo.get_user_id(refresh_token, &now).await?;

        let sub = user_id.to_string();
        let expires_in = Duration::days(14);
        let access_token = jwt::sign(&self.jwt_conf.secret.current, &sub, expires_in, now)?;
        let refresh_token = hex::encode(self.random_bytes_provider.get_bytes(32));
        let expires_at = now + expires_in;

        self.token_repo.update_token(&refresh_token, &expires_at).await?;

        Ok(AuthToken {
            expires_in: expires_in.num_seconds() as i32,
            access_token,
            refresh_token,
        })
    }
}
