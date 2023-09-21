use aws_lambda_events::event::sqs::SqsEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::info;

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    info!("{:?}", event);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("----- test 1 -----");
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).with_target(false).init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .json()
            .init();
    }
    println!("----- test 2 -----");

    info!("----- start -----");

    println!("----- test 3 -----");

    run(service_fn(function_handler)).await
}
