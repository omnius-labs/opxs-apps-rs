use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::shared::AppError;

use super::model::User;

#[async_trait]
pub trait UserRepo: Sync + Send + 'static {
    async fn create(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        salt: &str,
    ) -> Result<(), AppError>;
    async fn delete(&self, email: &str) -> Result<(), AppError>;
    async fn find_by_email(&self, email: &str) -> Result<User, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<User, AppError>;
}

#[async_trait]
pub trait RefreshTokenRepo: Sync + Send + 'static {
    async fn create(
        &self,
        user_id: &i64,
        token: &str,
        expires_at: &DateTime<Utc>,
    ) -> Result<(), AppError>;
}
