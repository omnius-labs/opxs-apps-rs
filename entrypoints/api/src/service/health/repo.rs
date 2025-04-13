use std::sync::Arc;

use sqlx::{PgPool, Row};

use crate::prelude::*;

pub struct WorldRepo {
    pub db: Arc<PgPool>,
}

impl WorldRepo {
    pub async fn get_mode(&self) -> Result<String> {
        let row = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'")
            .fetch_one(self.db.as_ref())
            .await?;

        row.try_get("value").map_err(|e| Error::new(ErrorKind::UnexpectedError).source(e))
    }
}
