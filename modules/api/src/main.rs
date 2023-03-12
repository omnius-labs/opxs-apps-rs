use std::sync::Arc;

use anyhow::anyhow;
use migration::Migrator;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    infra::secret::AwsSecretReader,
    shared::{AppConfig, AppState},
};

mod domain;
mod infra;
mod interface;
mod shared;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "opxs_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cmd = clap::Command::new("opxs-api")
        .bin_name("opxs-api")
        .arg(clap::arg!(--"mode" <MODE>));
    let matches = cmd.get_matches();
    let mode = matches
        .get_one::<String>("mode")
        .ok_or_else(|| anyhow!("'--mode' is not found"))?;

    info!("mode: {mode}");

    let path = format!("conf/{mode}.toml");
    let secret_reader = Arc::new(AwsSecretReader::new().await);
    let conf = AppConfig::load(&path, secret_reader).await?;

    let migrator = Migrator::new(&conf.postgres.url, "conf/migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(conf).await?;
    interface::WebServer::serve(&state).await?;

    Ok(())
}
