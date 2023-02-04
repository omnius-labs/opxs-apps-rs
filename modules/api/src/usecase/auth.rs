use std::sync::Arc;

use crate::{
    domain::auth::{repo::AuthRepo, service::AuthService},
    shared::AppError,
};

#[derive(Clone)]
pub struct AuthUseCase {
    pub auth_repo: Arc<dyn AuthRepo>,
}

impl AuthUseCase {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<(), AppError> {
        let service = AuthService {
            auth_repo: self.auth_repo.clone(),
        };

        service.register(name, email, password).await?;

        Ok(())
    }
}
