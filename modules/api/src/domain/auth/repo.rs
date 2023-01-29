use async_trait::async_trait;

use crate::shared::AppError;

use super::model::RegisterUser;

#[async_trait]
pub trait AuthRepo: Sync + Send + 'static {
    async fn register(&self, user: RegisterUser) -> Result<i64, AppError>;
}
