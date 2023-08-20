use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::auth::{model::User, repo::UserRepo},
    shared::AppError,
};

pub struct UserRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl UserRepo for UserRepoImpl {
    async fn get_user(&self, user_id: &i64) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT *
    FROM users
    WHERE id = $1;
"#,
        )
        .bind(user_id)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
