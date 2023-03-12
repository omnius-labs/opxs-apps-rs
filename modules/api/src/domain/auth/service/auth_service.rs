use std::sync::Arc;

use crate::{
    domain::auth::{model::User, repo::AuthRepo},
    shared::AppError,
};

use super::encrypted_password::EncryptedPassword;

#[allow(unused)]
pub struct AuthService {
    pub auth_repo: Arc<dyn AuthRepo>,
}

impl AuthService {
    #[allow(unused)]
    pub async fn register(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let encrypted_password =
            EncryptedPassword::new(password).map_err(AppError::UnexpectedError)?;

        let user = User {
            name: name.to_string(),
            email: email.to_string(),
            encrypted_password_hash: hex::encode(encrypted_password.hash),
            encrypted_password_salt: hex::encode(encrypted_password.salt),
        };

        self.auth_repo.create(&user).await?;

        Ok(user)
    }
}
