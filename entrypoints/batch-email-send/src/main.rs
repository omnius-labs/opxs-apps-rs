use std::sync::Arc;

use aws_config::BehaviorVersion;
use aws_lambda_events::event::sqs::SqsEvent;
use chrono::Duration;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;

use omnius_core_base::clock::ClockUtc;
use omnius_core_cloud::aws::ses::SesSenderImpl;

use omnius_opxs_base::{AppConfig, AppInfo, RunMode};
use omnius_opxs_email_send::{EmailSendExecutor, EmailSendJobBatchSqsMessage, EmailSendJobRepository};

const APP_NAME: &str = "opxs-batch-email-send";

async fn handler_sub(ms: &[EmailSendJobBatchSqsMessage]) -> Result<(), Error> {
    let mode = RunMode::from_env()?;
    let info = AppInfo::new(APP_NAME, mode)?;
    info!("info: {}", info);

    let conf = AppConfig::load(&info).await?;
    let db = Arc::new(
        PgPoolOptions::new()
            .max_connections(100)
            .idle_timeout(Some(Duration::minutes(15).to_std()?))
            .connect(&conf.postgres.url)
            .await?,
    );
    let clock = Arc::new(ClockUtc {});

    let executor = EmailSendExecutor {
        email_send_job_repository: Arc::new(EmailSendJobRepository { db: db.clone(), clock }),
        ses_sender: Arc::new(SesSenderImpl {
            client: aws_sdk_sesv2::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await),
            configuration_set_name: Some(
                conf.email
                    .ses
                    .ok_or_else(|| anyhow::anyhow!("ses config is not found"))?
                    .configuration_set_name,
            ),
        }),
    };
    executor.execute(ms).await?;

    Ok(())
}

async fn handler(event: LambdaEvent<serde_json::Value>) -> Result<(), Error> {
    let (event, _context) = event.into_parts();

    let mut ms: Vec<EmailSendJobBatchSqsMessage> = Vec::new();

    if let Ok(event) = serde_json::from_value::<SqsEvent>(event.clone()) {
        info!("sqs event");
        for v in event.records.into_iter().flat_map(|n| n.body).collect::<Vec<_>>() {
            let m = serde_json::from_str::<EmailSendJobBatchSqsMessage>(&v)?;
            ms.push(m);
        }
    } else {
        info!("raw event");
        let m = serde_json::from_value::<EmailSendJobBatchSqsMessage>(event)?;
        ms.push(m);
    }

    handler_sub(&ms).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    if cfg!(debug_assertions) {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();
    } else {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt().with_env_filter(filter).with_target(false).json().init();
    }

    info!("----- start -----");
    run(service_fn(handler)).await
}
