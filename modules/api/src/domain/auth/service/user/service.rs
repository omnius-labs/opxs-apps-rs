use std::sync::Arc;

use crate::{
    domain::auth::{model::User, repo::UserRepo},
    shared::AppError,
};

pub struct UserService {
    pub user_repo: Arc<dyn UserRepo + Send + Sync>,
}

impl UserService {
    pub async fn get_user(&self, user_id: &i64) -> Result<User, AppError> {
        let user = self.user_repo.get_user(user_id).await?;
        Ok(user)
    }
}
