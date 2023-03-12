use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::auth::{model::User, repo::AuthRepo},
    shared::AppError,
};

pub struct PostgresAuthRepo {
    pub db: PgPool,
}

#[async_trait]
impl AuthRepo for PostgresAuthRepo {
    async fn create(&self, user: &User) -> Result<i64, AppError> {
        if self.find_user_by_email(user.email.as_str()).await.is_ok() {
            return Err(AppError::UserAlreadyExists);
        }

        let result: (i64,) = sqlx::query_as(
            r#"
INSERT INTO users (name, email, encrypted_password_hash, encrypted_password_salt)
VALUES ($1, $2, $3, $4)
RETURNING id
"#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.encrypted_password_hash)
        .bind(&user.encrypted_password_salt)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(result.0)
    }
}

impl PostgresAuthRepo {
    async fn find_user_by_email(&self, email: &str) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT * FROM users WHERE email = $1
"#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
