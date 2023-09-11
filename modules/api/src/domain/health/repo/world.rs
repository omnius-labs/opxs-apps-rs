use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::shared::AppError;

#[async_trait]
pub trait WorldRepo {
    async fn get_mode(&self) -> Result<String, AppError>;
}

pub struct WorldRepoImpl {
    pub db: Arc<PgPool>,
}

#[async_trait]
impl WorldRepo for WorldRepoImpl {
    async fn get_mode(&self) -> Result<String, AppError> {
        let row = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'")
            .fetch_one(self.db.as_ref())
            .await
            .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(row.try_get("value").map_err(|e| AppError::UnexpectedError(e.into()))?)
    }
}
