use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi as _;

use omnius_core_base::clock::ClockUtc;
use omnius_core_migration::postgres::PostgresMigrator;

use omnius_opxs_base::{AppConfig, AppInfo, RunMode, WorldValidator};

use crate::{interface::ApiDoc, shared::state::AppState};

mod emulator;
mod error;
mod interface;
mod prelude;
mod result;
mod service;
mod shared;

const APP_NAME: &str = "opxs-api";

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "gen-openapi", value_name = "PATH")]
    gen_openapi: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(path) = args.gen_openapi {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = ApiDoc::openapi().to_yaml().unwrap();
        std::fs::write(&path, yaml)?;
        println!("openapi generated: {}", path.display());

        return Ok(());
    }

    if cfg!(debug_assertions) {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();
    } else {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt().with_env_filter(filter).with_target(false).json().init();
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

    let migrator = PostgresMigrator::new(&conf.postgres.url, "./conf/migrations", APP_NAME, "").await?;
    migrator.migrate().await?;

    let state = AppState::new(info, conf).await?;
    interface::WebServer::serve(state).await?;

    Ok(())
}
