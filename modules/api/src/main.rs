use std::{env, sync::Arc};

use tracing::info;

use migration::Migrator;

use crate::{
    infra::secret::AwsSecretReader,
    shared::{AppConfig, AppState},
};

mod domain;
mod infra;
mod interface;
mod shared;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .json()
            .init();
    }

    info!("----- start -----");

    let mode = env::var("RUN_MODE")?;
    info!("mode: {}", mode);
    info!("git_semver: {}", env!("VERGEN_GIT_SEMVER"));
    info!("git_sha: {}", env!("VERGEN_GIT_SHA"));
    info!("build_timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));

    let path = format!("conf/{mode}.toml");
    let secret_reader = Arc::new(AwsSecretReader::new().await);
    let conf = AppConfig::load(&path, secret_reader).await?;

    let migrator = Migrator::new(&conf.postgres.url, "./migrations", "opxs-api", "").await?;
    migrator.migrate().await?;

    let state = Arc::new(AppState::new(conf).await?);
    interface::WebServer::serve(&state).await?;

    Ok(())
}
