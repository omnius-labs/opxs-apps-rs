use std::sync::Arc;

use omnius_opxs_base::AppError;

use crate::model::User;

use super::UserRepo;

pub struct UserService {
    pub user_repo: Arc<UserRepo>,
}

impl UserService {
    pub async fn get_user(&self, user_id: &str) -> Result<User, AppError> {
        let user = self.user_repo.get_user(user_id).await?;
        Ok(user)
    }
}
