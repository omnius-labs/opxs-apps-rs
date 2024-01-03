use std::sync::Arc;

use sqlx::PgPool;

use crate::shared::{error::AuthError, model::User};

pub struct UserRepo {
    pub db: Arc<PgPool>,
}

impl UserRepo {
    pub async fn get_user(&self, user_id: &str) -> Result<User, AuthError> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT *
    FROM users
    WHERE id = $1;
"#,
        )
        .bind(user_id)
        .fetch_optional(self.db.as_ref())
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AuthError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
