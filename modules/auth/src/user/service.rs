use std::sync::Arc;

use crate::{model::User, prelude::*};

use super::UserRepo;

pub struct UserService {
    pub user_repo: Arc<UserRepo>,
}

impl UserService {
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let user = self.user_repo.get_user(user_id).await?;
        Ok(user)
    }
}
