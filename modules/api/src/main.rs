use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod domain;
mod infra;
mod interface;
mod provider;
mod usecase;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = provider::AppStateProvider::provide();
    let server = interface::WebServer::new(state);
    server.run().await?;

    Ok(())
}
