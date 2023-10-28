use tracing::info;

use core_cloud::aws::secrets::SecretsReaderImpl;
use core_migration::Migrator;

use crate::common::{AppConfig, AppInfo, AppState, WorldValidator};

mod common;
mod domain;
mod interface;
mod service;

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

    let secret_reader = Box::new(SecretsReaderImpl {
        client: aws_sdk_secretsmanager::Client::new(&aws_config::load_from_env().await),
    });
    let conf = AppConfig::load(&info.mode, secret_reader).await?;

    let world_verifier = WorldValidator {};
    world_verifier.verify(&info.mode, &conf.postgres.url).await?;

    let migrator = Migrator::new(&conf.postgres.url, "./migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
