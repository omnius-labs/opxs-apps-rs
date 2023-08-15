use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::{
    domain::auth::{model::User, repo::ProviderAuthRepo},
    shared::AppError,
};

pub struct ProviderAuthRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl ProviderAuthRepo for ProviderAuthRepoImpl {
    async fn create_user(&self, name: &str, provider_type: &str, provider_user_id: &str) -> Result<i64, AppError> {
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
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        tx.commit().await?;

        Ok(user_id)
    }

    async fn delete_user(&self, provider_type: &str, provider_user_id: &str) -> Result<(), AppError> {
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
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn exist_user(&self, provider_type: &str, provider_user_id: &str) -> Result<bool, AppError> {
        let row = sqlx::query(
            r#"
SELECT COUNT(1) FROM users u
    JOIN users_auth_provider p on u.id = p.user_id
    WHERE p.provider_type = $1 AND p.provider_user_id = $2
    LIMIT 1;
"#,
        )
        .bind(provider_type)
        .bind(provider_user_id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        let count = row.try_get::<i64, _>(0)?;

        Ok(count == 1)
    }

    async fn get_user(&self, provider_type: &str, provider_user_id: &str) -> Result<User, AppError> {
        let user: User = sqlx::query_as(
            r#"
SELECT * FROM users u
    JOIN users_auth_provider p on u.id = p.user_id
    WHERE p.provider_type = $1 AND p.provider_user_id = $2
    LIMIT 1;
"#,
        )
        .bind(provider_type)
        .bind(provider_user_id)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(user)
    }
}