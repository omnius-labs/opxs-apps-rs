use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{domain::auth::repo::RefreshTokenRepo, shared::AppError};

pub struct RefreshTokenRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl RefreshTokenRepo for RefreshTokenRepoImpl {
    async fn create(
        &self,
        user_id: &i64,
        token: &str,
        expires_at: &DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
INSERT INTO refresh_tokens (user_id, token, expires_at)
    VALUES ($1, $2, $3)
    RETURNING id
"#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(self.db.as_ref())
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(())
    }
}
