use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::{domain::auth::model::EmailUser, shared::AppError};

#[async_trait]
pub trait EmailAuthRepo {
    async fn create_user(&self, name: &str, email: &str, password_hash: &str, salt: &str) -> Result<i64, AppError>;
    async fn delete_user(&self, email: &str) -> Result<(), AppError>;
    async fn exist_user(&self, email: &str) -> Result<bool, AppError>;
    async fn get_user(&self, email: &str) -> Result<EmailUser, AppError>;
}

pub struct EmailAuthRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl EmailAuthRepo for EmailAuthRepoImpl {
    async fn create_user(&self, name: &str, email: &str, password_hash: &str, salt: &str) -> Result<i64, AppError> {
        let mut tx = self.db.begin().await?;

        let row = sqlx::query(
            r#"
INSERT INTO users (name, authentication_type)
    VALUES ($1, 'provider')
    RETURNING id;
"#,
        )
        .bind(name)
        .fetch_one(&mut tx)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        let user_id = row.try_get::<i64, _>(0)?;

        sqlx::query(
            r#"
INSERT INTO users_auth_email (user_id, email, password_hash, salt)
    VALUES ($1, $2, $3, $4)
    RETURNING id;
"#,
        )
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .bind(salt)
        .execute(&mut tx)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        tx.commit().await?;

        Ok(user_id)
    }

    async fn delete_user(&self, email: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
DELETE FROM users
    WHERE id = (SELECT user_id FROM users_auth_email WHERE email = $1);
"#,
        )
        .bind(email)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn exist_user(&self, email: &str) -> Result<bool, AppError> {
        let (existed,): (bool,) = sqlx::query_as(
            r#"
SELECT EXISTS (
    SELECT COUNT(1)
        FROM users u
        JOIN users_auth_email e on u.id = e.user_id
        WHERE e.email = $1
        LIMIT 1
);
"#,
        )
        .bind(email)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(existed)
    }

    async fn get_user(&self, email: &str) -> Result<EmailUser, AppError> {
        let user: Option<EmailUser> = sqlx::query_as(
            r#"
SELECT u.id, e.email, e.password_hash, e.salt, u.created_at, u.updated_at
    FROM users u
    JOIN users_auth_email e on u.id = e.user_id
    WHERE e.email = $1
    LIMIT 1;
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
}
