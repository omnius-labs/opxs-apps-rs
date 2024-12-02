use std::sync::Arc;

use tracing::info;

use omnius_core_base::clock::ClockUtc;
use omnius_core_migration::postgres::PostgresMigrator;

use omnius_opxs_base::{AppConfig, AppInfo, RunMode, WorldValidator};
use tracing_subscriber::EnvFilter;

use crate::shared::state::AppState;

mod emulator;
mod interface;
mod service;
mod shared;

const APP_NAME: &str = "opxs-api";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
    } else {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .json()
            .init();
    }

    info!("----- start -----");

    let mode = RunMode::from_env()?;
    let info = AppInfo::new(APP_NAME, mode)?;
    info!("info: {}", info);

    let conf = AppConfig::load(&info).await?;

    let clock = Arc::new(ClockUtc {});

    let world_verifier = WorldValidator::new(&info, &conf.postgres.url, clock).await?;
    world_verifier.verify().await?;

    if let Some(notify_conf) = &conf.notify {
        world_verifier.notify(notify_conf).await?;
    }

    let migrator =
        PostgresMigrator::new(&conf.postgres.url, "./conf/migrations", APP_NAME, "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
