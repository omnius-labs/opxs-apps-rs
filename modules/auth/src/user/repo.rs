use std::sync::Arc;

use sqlx::PgPool;

use crate::{model::User, prelude::*};

pub struct UserRepo {
    pub db: Arc<PgPool>,
}

impl UserRepo {
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let user: Option<User> = sqlx::query_as(
            r#"
SELECT *
    FROM users
    WHERE id = $1;
"#,
        )
        .bind(user_id)
        .fetch_optional(self.db.as_ref())
        .await?;

        if user.is_none() {
            return Err(Error::builder().kind(ErrorKind::NotFound).message("User not found").build());
        }

        Ok(user.unwrap())
    }
}
