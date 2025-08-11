use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use omnius_core_base::clock::Clock;

use crate::{model::User, prelude::*};

pub struct TokenRepo {
    pub db: Arc<PgPool>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
}

impl TokenRepo {
    pub async fn create_token(&self, user_id: &str, refresh_token: &str, refresh_token_expires_at: &DateTime<Utc>) -> Result<()> {
        let now = self.clock.now();
        sqlx::query(
            r#"
INSERT INTO refresh_tokens (refresh_token, user_id, expires_at, created_at)
    VALUES ($1, $2, $3, $4);
"#,
        )
        .bind(refresh_token)
        .bind(user_id)
        .bind(refresh_token_expires_at)
        .bind(now)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn create_token_and_delete_token(
        &self,
        user_id: &str,
        refresh_token: &str,
        refresh_token_expires_at: &DateTime<Utc>,
        old_refresh_token: &str,
    ) -> Result<()> {
        let now = self.clock.now();

        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
INSERT INTO refresh_tokens (refresh_token, user_id, expires_at, created_at)
    VALUES ($1, $2, $3, $4);
"#,
        )
        .bind(refresh_token)
        .bind(user_id)
        .bind(refresh_token_expires_at)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
DELETE FROM refresh_tokens
    WHERE user_id = $1;
"#,
        )
        .bind(old_refresh_token)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_token(&self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
DELETE FROM refresh_tokens
    WHERE user_id = $1;
"#,
        )
        .bind(user_id)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_user_id(&self, refresh_token: &str) -> Result<String> {
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
        .await?;

        if user.is_none() {
            return Err(Error::builder().kind(ErrorKind::NotFound).message("User not found").build());
        }

        Ok(user.unwrap().id)
    }
}
