use std::sync::Arc;

use chrono::Duration;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::{service::AppService, AppConfig};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub conf: AppConfig,
    pub service: AppService,
}

impl AppState {
    pub async fn new(conf: AppConfig) -> anyhow::Result<Self> {
        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&conf.postgres.url)
                .await?,
        );
        let service = AppService::new(&db, &conf);

        Ok(Self { conf, db, service })
    }
}
