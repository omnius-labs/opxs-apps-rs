use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use omnius_core_base::clock::Clock;

use omnius_opxs_base::AppError;

use crate::shared::model::User;

pub struct TokenRepo {
    pub db: Arc<PgPool>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
}

impl TokenRepo {
    pub async fn create_token(
        &self,
        user_id: &str,
        refresh_token: &str,
        refresh_token_expires_at: &DateTime<Utc>,
    ) -> Result<(), AppError> {
        let now = self.clock.now();
        sqlx::query(
            r#"
INSERT INTO refresh_tokens (refresh_token, user_id, expires_at, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5);
"#,
        )
        .bind(refresh_token)
        .bind(user_id)
        .bind(refresh_token_expires_at)
        .bind(now)
        .bind(now)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    pub async fn delete_token(&self, user_id: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
DELETE FROM refresh_tokens
    WHERE user_id = $1;
"#,
        )
        .bind(user_id)
        .execute(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }

    pub async fn get_user_id(&self, refresh_token: &str) -> Result<String, AppError> {
        let now = self.clock.now();
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT u.*
    FROM users u
    JOIN refresh_tokens t on t.user_id = u.id
    WHERE t.refresh_token = $1 AND expires_at > $2;
"#,
        )
        .bind(refresh_token)
        .bind(now)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::RefreshTokenNotFound);
        }

        Ok(user.unwrap().id)
    }
}
