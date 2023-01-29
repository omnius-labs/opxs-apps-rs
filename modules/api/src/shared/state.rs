use sqlx::{postgres::PgPoolOptions, PgPool};

use super::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub async fn new(conf: &AppConfig) -> anyhow::Result<AppState> {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&conf.database_url)
            .await?;

        let state = AppState { db };

        Ok(state)
    }
}
