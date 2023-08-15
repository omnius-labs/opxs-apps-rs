use std::sync::Arc;

use chrono::{Duration, Utc};

use omnius_core_base::{clock::SystemClock, random_string::RandomStringGenerator};

use crate::{
    domain::auth::{
        model::{AuthToken, User},
        repo::TokenRepo,
    },
    shared::{AppError, JwtConfig},
};

use super::jwt;

pub struct TokenService {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub token_generator: Arc<dyn RandomStringGenerator + Send + Sync>,
    pub jwt_conf: JwtConfig,
    pub token_repo: Arc<dyn TokenRepo + Send + Sync>,
}

impl TokenService {
    pub async fn create(&self, user_id: &i64) -> Result<AuthToken, AppError> {
        let expires_in = Duration::days(14);
        let access_token = jwt::sign(
            &self.jwt_conf.secret.current,
            user_id.to_string().as_str(),
            expires_in,
            &self.system_clock,
        )?;
        let refresh_token = self.token_generator.gen();
        let expires_at = self.system_clock.now() + expires_in;

        self.token_repo.create_token(user_id, &refresh_token, &expires_at).await?;

        Ok(AuthToken {
            expires_in: expires_in.num_seconds() as i32,
            access_token,
            refresh_token,
        })
    }

    // pub async fn delete(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }

    // pub async fn refresh(&self, refresh_token: &str) -> Result<AuthToken, AppError> {
    //     todo!()
    // }

    pub async fn get_user(&self, access_token: &str) -> Result<User, AppError> {
        let claims = jwt::verify(&self.jwt_conf.secret.current, access_token)?;
        let user_id = claims.sub.parse::<i64>().map_err(|e| AppError::UnexpectedError(e.into()))?;
        let user = self.token_repo.get_user(user_id).await?;
        Ok(user)
    }
}
