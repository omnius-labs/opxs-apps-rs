use aws_lambda_events::event::sns::SnsEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::info;

async fn function_handler(_event: LambdaEvent<SnsEvent>) -> Result<(), Error> {
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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

    run(service_fn(function_handler)).await
}
