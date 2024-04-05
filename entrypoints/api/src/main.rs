use std::sync::Arc;

use tracing::info;

use core_base::clock::SystemClockUtc;
use core_migration::postgres::PostgresMigrator;

use opxs_base::{AppConfig, AppInfo, WorldValidator};
use tracing_subscriber::EnvFilter;

use crate::shared::state::AppState;

mod interface;
mod service;
mod shared;

const APPLICATION_NAME: &str = "opxs-api";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).with_target(false).init();
    } else {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt().with_env_filter(filter).with_target(false).json().init();
    }

    info!("----- start -----");

    let info = AppInfo::new()?;
    info!("info: {}", info);

    let conf = AppConfig::load(APPLICATION_NAME, &info.mode).await?;

    let system_clock = Arc::new(SystemClockUtc {});
    let world_verifier = WorldValidator::new(info.clone(), &conf.postgres.url, system_clock).await?;
    world_verifier.verify().await?;
    world_verifier.notify(&conf.notify).await?;

    let migrator = PostgresMigrator::new(&conf.postgres.url, "./conf/migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
