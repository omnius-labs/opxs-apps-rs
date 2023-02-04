use std::sync::Arc;

use crate::{
    domain::auth::{model::User, repo::AuthRepo},
    shared::AppError,
};

use super::password_deriver::{PasswordDerived, PasswordDeriver};

pub struct AuthService {
    pub auth_repo: Arc<dyn AuthRepo>,
}

impl AuthService {
    pub async fn register(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let derived: PasswordDerived = PasswordDeriver::derive(password)?;

        let user = User {
            name: name.to_string(),
            email: email.to_string(),
            password: hex::encode(derived.hash),
            salt: hex::encode(derived.salt),
        };

        self.auth_repo.create(&user).await?;

        Ok(user)
    }
}
