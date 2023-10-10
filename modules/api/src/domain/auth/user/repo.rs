use std::sync::Arc;

use sqlx::PgPool;

use crate::{common::AppError, domain::auth::model::User};

pub struct UserRepo {
    pub db: Arc<PgPool>,
}

impl UserRepo {
    pub async fn get_user(&self, user_id: &i64) -> Result<User, AppError> {
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
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        Ok(user.unwrap())
    }
}
