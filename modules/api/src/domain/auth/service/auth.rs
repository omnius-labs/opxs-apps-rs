use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    domain::auth::{
        model::{AuthToken, User},
        repo::{RefreshTokenRepo, UserRepo},
    },
    shared::{AppError, JwtConfig},
};

mod jwt;
mod kdf;

#[derive(Clone)]
pub struct AuthService {
    pub jwt_conf: JwtConfig,
    pub user_repo: Arc<dyn UserRepo>,
    pub refresh_token_repo: Arc<dyn RefreshTokenRepo>,
}

impl AuthService {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<(), AppError> {
        if self.user_repo.find_by_name(name).await.is_ok() {
            return Err(AppError::DuplicateUserName);
        }
        if self.user_repo.find_by_email(email).await.is_ok() {
            return Err(AppError::DuplicateUserEmail);
        }

        let salt = kdf::salt()?;
        let password_hash = kdf::derive(password, &salt)?;

        self.user_repo
            .create(name, email, &hex::encode(password_hash), &hex::encode(salt))
            .await?;

        Ok(())
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthToken, AppError> {
        let user = self.user_repo.find_by_email(email).await?;
        let salt = hex::decode(user.salt).map_err(|e| AppError::UnexpectedError(e.into()))?;
        let password_hash =
            hex::decode(user.password_hash).map_err(|e| AppError::UnexpectedError(e.into()))?;

        if !kdf::verify(password, &salt, &password_hash)? {
            return Err(AppError::WrongPassword);
        }

        let access_token = jwt::sign(&self.jwt_conf.secret, email, Duration::hours(1))?;
        let refresh_token = Uuid::new_v4().simple().to_string();
        let expires_at = Utc::now() + Duration::days(14);

        self.refresh_token_repo
            .create(&user.id, &refresh_token, &expires_at)
            .await?;

        Ok(AuthToken {
            expires_in: 3600,
            access_token,
            refresh_token,
        })
    }

    // pub async fn refresh(&self, refresh_token: &str) -> Result<AuthToken, AppError> {
    //     todo!()
    // }

    // pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }

    pub async fn verify(&self, access_token: &str) -> Result<User, AppError> {
        let claims = jwt::verify(&self.jwt_conf.secret, access_token)?;
        let email = claims.sub;
        let user = self.user_repo.find_by_email(&email).await?;
        Ok(user)
    }
}
