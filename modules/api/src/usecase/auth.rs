use std::sync::Arc;

use crate::{
    domain::auth::{model::RegisterUser, repo::AuthRepo},
    shared::AppError,
};

#[derive(Clone)]
pub struct AuthUseCase {
    pub auth_repo: Arc<dyn AuthRepo>,
}

impl AuthUseCase {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<i64, AppError> {
        let id = self
            .auth_repo
            .register(RegisterUser {
                name: name.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            })
            .await?;

        Ok(id)
    }
}
