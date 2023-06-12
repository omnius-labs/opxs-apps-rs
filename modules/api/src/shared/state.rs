use std::sync::Arc;

use chrono::Duration;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::{service::AppService, AppConfig, AppInfo};

#[derive(Clone)]
pub struct AppState {
    pub info: AppInfo,
    pub conf: AppConfig,
    pub db: Arc<PgPool>,
    pub service: AppService,
}

impl AppState {
    pub async fn new(info: AppInfo, conf: AppConfig) -> anyhow::Result<Self> {
        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&conf.postgres.url)
                .await?,
        );
        let service = AppService::new(&info, &conf, &db);

        Ok(Self {
            info,
            conf,
            db,
            service,
        })
    }
}
