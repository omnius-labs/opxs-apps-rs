use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use sqlx::PgPool;

use omnius_core_base::{clock::Clock, tsid::TsidProvider};

use omnius_opxs_base::AppError;

use crate::shared::model::{User, UserAuthenticationType, UserRole};

pub struct ProviderAuthRepo {
    pub db: Arc<PgPool>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub tsid_provider: Arc<Mutex<dyn TsidProvider + Send + Sync>>,
}

impl ProviderAuthRepo {
    pub async fn create_user(
        &self,
        name: &str,
        provider_type: &str,
        provider_user_id: &str,
    ) -> Result<String, AppError> {
        let user_id = self.tsid_provider.lock().gen().to_string();
        let now = self.clock.now();

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
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

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
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        tx.commit().await?;

        Ok(user_id)
    }

    pub async fn delete_user(&self, id: &str) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;

        let queries = vec![
            sqlx::query("DELETE FROM users WHERE id = $1").bind(id),
            sqlx::query("DELETE FROM user_auth_providers WHERE user_id = $1").bind(id),
            sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1").bind(id),
        ];

        for query in queries {
            query
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::UnexpectedError(e.into()))?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn exist_user(
        &self,
        provider_type: &str,
        provider_user_id: &str,
    ) -> Result<bool, AppError> {
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
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(existed)
    }

    pub async fn get_user(
        &self,
        provider_type: &str,
        provider_user_id: &str,
    ) -> Result<User, AppError> {
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
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        let user = user.ok_or_else(|| AppError::UserNotFound)?;
        Ok(user)
    }
}
