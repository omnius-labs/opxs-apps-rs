use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    domain::auth::{model::User, repo::TokenRepo},
    shared::AppError,
};

pub struct TokenRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl TokenRepo for TokenRepoImpl {
    async fn create_token(&self, user_id: &i64, refresh_token: &str, expires_at: &DateTime<Utc>) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO users_tokens (user_id, refresh_token, expires_at)
    VALUES ($1, $2, $3)
"#,
        )
        .bind(user_id)
        .bind(refresh_token)
        .bind(expires_at)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn get_user(&self, user_id: i64) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT *
    FROM users
    WHERE user.id = $1
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
