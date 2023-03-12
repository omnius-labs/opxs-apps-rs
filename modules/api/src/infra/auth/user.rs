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
    async fn create(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        salt: &str,
    ) -> Result<(), AppError> {
        if self.find_by_email(email).await.is_ok() {
            return Err(AppError::DuplicateUserEmail);
        }

        sqlx::query(
            r#"
INSERT INTO users (name, email, password_hash, salt)
    VALUES ($1, $2, $3, $4)
    RETURNING id
"#,
        )
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .bind(salt)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn delete(&self, email: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
DELETE FROM users
    WHERE email = $1
"#,
        )
        .bind(email)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT * FROM users
    WHERE email = $1
"#,
        )
        .bind(email)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        Ok(user.unwrap())
    }

    async fn find_by_name(&self, name: &str) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT * FROM users
    WHERE name = $1
"#,
        )
        .bind(name)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
