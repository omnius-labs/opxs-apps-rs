use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::shared::AppError;

use super::model::{EmailUser, User};

#[async_trait]
pub trait EmailAuthRepo {
    async fn create_user(&self, name: &str, email: &str, password_hash: &str, salt: &str) -> Result<i64, AppError>;
    async fn delete_user(&self, email: &str) -> Result<(), AppError>;
    async fn exist_user(&self, email: &str) -> Result<bool, AppError>;
    async fn get_user(&self, email: &str) -> Result<EmailUser, AppError>;
}

#[async_trait]
pub trait ProviderAuthRepo {
    async fn create_user(&self, name: &str, provider_type: &str, provider_user_id: &str) -> Result<i64, AppError>;
    async fn delete_user(&self, provider_type: &str, provider_user_id: &str) -> Result<(), AppError>;
    async fn exist_user(&self, provider_type: &str, provider_user_id: &str) -> Result<bool, AppError>;
    async fn get_user(&self, provider_type: &str, provider_user_id: &str) -> Result<User, AppError>;
}

#[async_trait]
pub trait TokenRepo {
    async fn create_token(&self, user_id: &i64, refresh_token: &str, expires_at: &DateTime<Utc>) -> Result<(), AppError>;
    async fn get_user(&self, user_id: i64) -> Result<User, AppError>;
}
