use async_trait::async_trait;

use crate::shared::AppError;

#[async_trait]
pub trait WorldRepo {
    async fn get_mode(&self) -> Result<String, AppError>;
}
