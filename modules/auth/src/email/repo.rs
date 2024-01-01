use std::sync::Arc;

use sqlx::PgPool;

use crate::shared::{
    error::AuthError,
    model::{EmailUser, UserAuthenticationType, UserRole},
};

pub struct EmailAuthRepo {
    pub db: Arc<PgPool>,
}

impl EmailAuthRepo {
    pub async fn create_user(&self, name: &str, email: &str, password_hash: &str, salt: &str) -> Result<i64, AuthError> {
        let mut tx = self.db.begin().await?;

        let (user_id,): (i64,) = sqlx::query_as(
            r#"
INSERT INTO users (name, authentication_type, role)
    VALUES ($1, $2, $3)
    RETURNING id;
"#,
        )
        .bind(name)
        .bind(UserAuthenticationType::Email)
        .bind(UserRole::User)
        .fetch_one(&mut tx)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        sqlx::query(
            r#"
INSERT INTO user_auth_emails (user_id, email, password_hash, salt)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (email)
    DO UPDATE SET
        user_id = $1,
        password_hash = $3,
        salt = $4
    RETURNING id;
"#,
        )
        .bind(user_id)
        .bind(email)
        .bind(password_hash)
        .bind(salt)
        .execute(&mut tx)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        tx.commit().await?;

        Ok(user_id)
    }

    pub async fn delete_user(&self, email: &str) -> Result<(), AuthError> {
        sqlx::query(
            r#"
DELETE FROM users
    WHERE id = (SELECT user_id FROM user_auth_emails WHERE email = $1);
"#,
        )
        .bind(email)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        Ok(())
    }

    pub async fn exist_user(&self, email: &str) -> Result<bool, AuthError> {
        let (existed,): (bool,) = sqlx::query_as(
            r#"
SELECT EXISTS (
    SELECT u.id
        FROM users u
        JOIN user_auth_emails e on u.id = e.user_id
        WHERE e.email = $1 AND e.email_verified = true
        LIMIT 1
);
"#,
        )
        .bind(email)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        Ok(existed)
    }

    pub async fn get_user(&self, email: &str) -> Result<EmailUser, AuthError> {
        let user: Option<EmailUser> = sqlx::query_as(
            r#"
SELECT u.id, u.name, u.role, e.email, e.password_hash, e.salt, u.created_at, u.updated_at
    FROM users u
    JOIN user_auth_emails e on u.id = e.user_id
    WHERE e.email = $1 AND e.email_verified = true
    LIMIT 1;
"#,
        )
        .bind(email)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AuthError::UserNotFound);
        }

        Ok(user.unwrap())
    }

    pub async fn update_email_verified(&self, email: &str, email_verified: bool) -> Result<(), AuthError> {
        sqlx::query(
            r#"
UPDATE user_auth_emails
    SET email_verified = $2
    WHERE email = $1;
"#,
        )
        .bind(email)
        .bind(email_verified)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        Ok(())
    }
}
