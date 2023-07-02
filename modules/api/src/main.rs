use tracing::info;

use omnius_core_cloud::secret::aws::AwsSecretReader;
use omnius_core_migration::Migrator;

use crate::shared::{AppConfig, AppInfo, AppState};

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

    let secret_reader = Box::new(AwsSecretReader::new().await?);
    let conf_path = format!("conf/{mode}.toml", mode = info.mode);
    let conf = AppConfig::load(&conf_path, secret_reader).await?;

    let migrator = Migrator::new(&conf.postgres.url, "./migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
