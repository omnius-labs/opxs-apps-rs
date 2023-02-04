use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::auth::{model::User, repo::AuthRepo},
    shared::AppError,
};

pub struct AuthRepoImpl {
    pub db: PgPool,
}

#[async_trait]
impl AuthRepo for AuthRepoImpl {
    async fn create(&self, user: &User) -> Result<i64, AppError> {
        if let Ok(_) = self.find_user_by_email(user.email.as_str()).await {
            return Err(AppError::UserAlreadyExists);
        }

        let result: (i64,) = sqlx::query_as(
            r#"
INSERT INTO users (name, email, password, salt)
VALUES ($1, $2, $3, $4)
RETURNING id
"#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .bind(&user.salt)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(result.0)
    }
}

impl AuthRepoImpl {
    async fn find_user_by_email(&self, email: &str) -> Result<User, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
select * from users where email = $1
"#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        if user.is_none() {
            return Err(AppError::UserNotFound);
        }

        return Ok(user.unwrap());
    }
}
