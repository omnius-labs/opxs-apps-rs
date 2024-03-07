use std::sync::Arc;

use sqlx::{PgPool, Row};

use opxs_base::AppError;

pub struct WorldRepo {
    pub db: Arc<PgPool>,
}

impl WorldRepo {
    pub async fn get_mode(&self) -> Result<String, AppError> {
        let row = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'")
            .fetch_one(self.db.as_ref())
            .await
            .map_err(|e| AppError::UnexpectedError(e.into()))?;

        row.try_get("value").map_err(|e| AppError::UnexpectedError(e.into()))
    }
}
