use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    domain::auth::{model::RegisterUser, repo::AuthRepo},
    shared::AppError,
};

pub struct AuthRepoImpl {
    pub db: PgPool,
}

#[async_trait]
impl AuthRepo for AuthRepoImpl {
    async fn register(&self, user: RegisterUser) -> Result<i64, AppError> {
        if let Ok(_) = self.find_by_email(user.email.as_str()).await {
            return Err(AppError::UserAlreadyExists);
        }

        let result: (i64,) = sqlx::query_as(
            r#"
INSERT INTO users (name, email, password)
VALUES ($1, $2, $3)
RETURNING id
"#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(result.0)
    }
}

impl AuthRepoImpl {
    async fn find_by_email(&self, email: &str) -> Result<RegisterUser, AppError> {
        let user: Option<RegisterUser> = sqlx::query_as(
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
