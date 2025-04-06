use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie;
use chrono::Duration;
use sqlx::{PgPool, postgres::PgPoolOptions};

use omnius_opxs_base::{AppConfig, AppInfo, RunMode};

use crate::Result;

use super::service::AppService;

#[derive(Clone)]
pub struct AppState {
    pub info: AppInfo,
    pub conf: AppConfig,
    pub db: Arc<PgPool>,
    pub service: Arc<AppService>,
    cookie_key: cookie::Key,
}

impl AppState {
    pub async fn new(info: AppInfo, conf: AppConfig) -> Result<Self> {
        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std()?))
                .connect(&conf.postgres.url)
                .await?,
        );

        let service = Arc::new(match info.mode {
            RunMode::Local => AppService::new_for_local(&info, &conf, db.clone()).await?,
            _ => AppService::new_for_cloud(&info, &conf, db.clone()).await?,
        });

        Ok(Self {
            info,
            conf,
            db,
            service,
            cookie_key: cookie::Key::generate(),
        })
    }
}

impl FromRef<AppState> for cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
