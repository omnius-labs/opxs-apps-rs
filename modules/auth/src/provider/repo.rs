use std::sync::Arc;

use chrono::Utc;
use core_base::{clock::SystemClock, tsid::TsidProvider};
use sqlx::PgPool;

use crate::shared::{
    error::AuthError,
    model::{User, UserAuthenticationType, UserRole},
};

pub struct ProviderAuthRepo {
    pub db: Arc<PgPool>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub tsid_provider: Arc<dyn TsidProvider + Send + Sync>,
}

impl ProviderAuthRepo {
    pub async fn create_user(&self, name: &str, provider_type: &str, provider_user_id: &str) -> Result<String, AuthError> {
        let user_id = self.tsid_provider.gen().to_string();
        let now = self.system_clock.now();

        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
INSERT INTO users (id, name, authentication_type, role, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6)
"#,
        )
        .bind(&user_id)
        .bind(name)
        .bind(UserAuthenticationType::Provider)
        .bind(UserRole::User)
        .bind(now)
        .bind(now)
        .execute(&mut tx)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        sqlx::query(
            r#"
INSERT INTO user_auth_providers (user_id, provider_type, provider_user_id, created_at)
    VALUES ($1, $2, $3, $4)
"#,
        )
        .bind(&user_id)
        .bind(provider_type)
        .bind(provider_user_id)
        .bind(now)
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
    WHERE id = (SELECT user_id FROM user_auth_providers WHERE provider_type = $1 AND provider_user_id = $2);
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
        JOIN user_auth_providers p on u.id = p.user_id
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
    JOIN user_auth_providers p on u.id = p.user_id
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
