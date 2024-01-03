use std::sync::Arc;

use crate::shared::{error::AuthError, model::User};

use super::UserRepo;

pub struct UserService {
    pub user_repo: Arc<UserRepo>,
}

impl UserService {
    pub async fn get_user(&self, user_id: &str) -> Result<User, AuthError> {
        let user = self.user_repo.get_user(user_id).await?;
        Ok(user)
    }
}
