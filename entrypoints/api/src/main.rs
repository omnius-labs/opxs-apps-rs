use std::sync::Arc;

use core_base::clock::SystemClockUtc;
use tracing::info;

use core_migration::postgres::PostgresMigrator;

use crate::shared::{config::AppConfig, info::AppInfo, state::AppState, world::WorldValidator};

mod domain;
mod interface;
mod shared;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).with_target(false).init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .json()
            .init();
    }

    info!("----- start -----");

    let info = AppInfo::new()?;
    info!("info: {}", info);

    let conf = AppConfig::load(&info.mode).await?;

    let system_clock = Arc::new(SystemClockUtc {});
    let world_verifier = WorldValidator { system_clock };
    world_verifier.verify(&info.mode, &conf.postgres.url).await?;

    let migrator = PostgresMigrator::new(&conf.postgres.url, "./conf/migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
