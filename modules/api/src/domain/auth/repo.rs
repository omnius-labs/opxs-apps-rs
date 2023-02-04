use async_trait::async_trait;

use crate::shared::AppError;

use super::model::User;

#[async_trait]
pub trait AuthRepo: Sync + Send + 'static {
    async fn create(&self, user: &User) -> Result<i64, AppError>;
}
