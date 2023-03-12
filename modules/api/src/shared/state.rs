use chrono::Duration;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub conf: AppConfig,
    pub db: PgPool,
}

impl AppState {
    pub async fn new(conf: AppConfig) -> anyhow::Result<AppState> {
        let db = PgPoolOptions::new()
            .max_connections(100)
            .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
            .connect(&conf.postgres.url)
            .await?;

        let state = AppState { conf, db };

        Ok(state)
    }
}
