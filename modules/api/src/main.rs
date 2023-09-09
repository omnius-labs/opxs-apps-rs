use tracing::info;

use omnius_core_cloud::aws::secrets::SecretsReaderImpl;
use omnius_core_migration::Migrator;

use crate::shared::{AppConfig, AppInfo, AppState, WorldValidator};

mod domain;
mod infra;
mod interface;
mod shared;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();
    } else {
        tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).json().init();
    }

    info!("----- start -----");

    let info = AppInfo::new()?;
    info!("{}", info);

    let sdk_config = aws_config::load_from_env().await;
    let secret_reader = Box::new(SecretsReaderImpl::new(sdk_config).await?);
    let conf_path = format!("conf/{mode}.toml", mode = info.mode);
    let conf = AppConfig::load(&conf_path, secret_reader).await?;

    let world_verifier = WorldValidator {};
    world_verifier.verify(&info.mode, &conf.postgres.url).await?;

    let migrator = Migrator::new(&conf.postgres.url, "./migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
