use std::sync::Arc;

use sqlx::PgPool;

use crate::shared::{
    error::AuthError,
    model::{User, UserAuthenticationType, UserRole},
};

pub struct ProviderAuthRepo {
    pub db: Arc<PgPool>,
}

impl ProviderAuthRepo {
    pub async fn create_user(&self, name: &str, provider_type: &str, provider_user_id: &str) -> Result<i64, AuthError> {
        let mut tx = self.db.begin().await?;

        let (user_id,): (i64,) = sqlx::query_as(
            r#"
INSERT INTO users (name, authentication_type, role)
    VALUES ($1, $2, $3)
    RETURNING id;
"#,
        )
        .bind(name)
        .bind(UserAuthenticationType::Provider)
        .bind(UserRole::User)
        .fetch_one(&mut tx)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        sqlx::query(
            r#"
INSERT INTO users_auth_provider (user_id, provider_type, provider_user_id)
    VALUES ($1, $2, $3)
    RETURNING id;
"#,
        )
        .bind(user_id)
        .bind(provider_type)
        .bind(provider_user_id)
        .execute(&mut tx)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        tx.commit().await?;

        Ok(user_id)
    }

    pub async fn delete_user(&self, provider_type: &str, provider_user_id: &str) -> Result<(), AuthError> {
        sqlx::query(
            r#"
DELETE FROM users
    WHERE id = (SELECT user_id FROM users_auth_provider WHERE provider_type = $1 AND provider_user_id = $2);
"#,
        )
        .bind(provider_type)
        .bind(provider_user_id)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        Ok(())
    }

    pub async fn exist_user(&self, provider_type: &str, provider_user_id: &str) -> Result<bool, AuthError> {
        let (existed,): (bool,) = sqlx::query_as(
            r#"
SELECT EXISTS (
    SELECT u.id
        FROM users u
        JOIN users_auth_provider p on u.id = p.user_id
        WHERE p.provider_type = $1 AND p.provider_user_id = $2
        LIMIT 1
);
"#,
        )
        .bind(provider_type)
        .bind(provider_user_id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        Ok(existed)
    }

    pub async fn get_user(&self, provider_type: &str, provider_user_id: &str) -> Result<User, AuthError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT *
    FROM users u
    JOIN users_auth_provider p on u.id = p.user_id
    WHERE p.provider_type = $1 AND p.provider_user_id = $2
    LIMIT 1;
"#,
        )
        .bind(provider_type)
        .bind(provider_user_id)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AuthError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
