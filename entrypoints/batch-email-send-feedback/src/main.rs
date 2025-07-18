use aws_lambda_events::event::sns::SnsEvent;
use lambda_runtime::{LambdaEvent, run, service_fn};
use tracing::info;
use tracing_subscriber::EnvFilter;

use omnius_opxs_base::{AppInfo, RunMode};

use omnius_opxs_email_send::SesNotification;

const APP_NAME: &str = "opxs-batch-email-send-feedback";

async fn handler_sub(_ms: &[SesNotification]) -> Result<(), lambda_runtime::Error> {
    let mode = RunMode::from_env()?;
    let info = AppInfo::new(APP_NAME, mode)?;
    info!("info: {}", info);

    todo!();

    // let sdk_config = aws_config::load_from_env().await;
    // let secret_reader = Box::new(SecretsReaderImpl {
    //     client: aws_sdk_secretsmanager::Client::new(&sdk_config),
    // });
    // let conf = AppConfig::load(&info.mode, secret_reader).await?;
    // let db = Arc::new(
    //     PgPoolOptions::new()
    //         .max_connections(100)
    //         .idle_timeout(Some(Duration::minutes(15).to_std()?))
    //         .connect(&conf.postgres.url)
    //         .await?,
    // );
    // let clock = Arc::new(ClockUtc {});
    // let tsid_provider = Arc::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl, 16));

    // let executor = Executor {
    //     email_send_job_repository: Arc::new(EmailSendJobRepository {
    //         db: db.clone(),
    //         clock,
    //         tsid_provider,
    //     }),
    //     ses_sender: Arc::new(SesSenderImpl {
    //         client: aws_sdk_sesv2::Client::new(&aws_config::load_from_env().await),
    //         configuration_set_name: Some(conf.ses.configuration_set_name),
    //     }),
    // };
    // executor.execute(ms).await?;

    // Ok(())
}

async fn handler(event: LambdaEvent<serde_json::Value>) -> Result<(), lambda_runtime::Error> {
    let (event, _context) = event.into_parts();

    let mut ms: Vec<SesNotification> = Vec::new();

    if let Ok(event) = serde_json::from_value::<SnsEvent>(event.clone()) {
        info!("sqs event");
        for v in event.records.into_iter().map(|n| n.sns.message).collect::<Vec<_>>() {
            let m = serde_json::from_str::<SesNotification>(&v)?;
            ms.push(m);
        }
    } else {
        info!("raw event");
        let m = serde_json::from_value::<SesNotification>(event)?;
        ms.push(m);
    }

    info!("messages: {:?}", ms);
    handler_sub(&ms).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
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
