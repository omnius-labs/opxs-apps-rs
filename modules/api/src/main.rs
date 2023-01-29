mod domain;
mod infra;
mod interface;
mod shared;
mod usecase;

use migration::Migrator;
use shared::{AppConfig, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "pxtv_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conf = AppConfig::load("conf/settings.toml");

    let migrator = Migrator::new(&conf.database_url, "conf/migrations", "pxtv-api", "").await?;
    migrator.migrate().await?;

    let state = AppState::new(&conf).await?;

    interface::WebServer::serve(&state).await?;

    Ok(())
}
