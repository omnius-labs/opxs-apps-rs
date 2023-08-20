use std::sync::Arc;

use crate::{
    domain::auth::{repo::EmailAuthRepo, service::Kdf},
    shared::{AppError, JwtConfig},
};

#[derive(Clone)]
pub struct EmailAuthService {
    pub auth_repo: Arc<dyn EmailAuthRepo + Send + Sync>,
    pub jwt_conf: JwtConfig,
    pub kdf: Kdf,
}

impl EmailAuthService {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<i64, AppError> {
        if self.auth_repo.exist_user(email).await? {
            return Err(AppError::DuplicateEmail);
        }

        let salt = self.kdf.gen_salt()?;
        let password_hash = self.kdf.derive(password, &salt)?;

        let user_id = self
            .auth_repo
            .create_user(name, email, &hex::encode(password_hash), &hex::encode(salt))
            .await?;

        Ok(user_id)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<i64, AppError> {
        if !self.auth_repo.exist_user(email).await? {
            return Err(AppError::WrongPassword);
        }

        let user = self.auth_repo.get_user(email).await?;
        let salt = hex::decode(user.salt).map_err(|e| AppError::UnexpectedError(e.into()))?;
        let password_hash = hex::decode(user.password_hash).map_err(|e| AppError::UnexpectedError(e.into()))?;

        if !self.kdf.verify(password, &salt, &password_hash)? {
            return Err(AppError::WrongPassword);
        }

        Ok(user.id)
    }

    // pub async fn unregister(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }
}
