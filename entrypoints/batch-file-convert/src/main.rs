use std::{path::Path, sync::Arc};

use aws_config::BehaviorVersion;
use aws_lambda_events::sqs::SqsEvent;
use chrono::Duration;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use parking_lot::Mutex;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::EnvFilter;

use omnius_core_base::{clock::ClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
use omnius_core_cloud::aws::s3::S3ClientImpl;

use omnius_opxs_base::{AppConfig, AppInfo, RunMode};
use omnius_opxs_file_convert::{FileConvertExecutor, FileConvertJobRepository, ImageConvertJobSqsMessage, ImageConverterImpl};

const APP_NAME: &str = "opxs-batch-file-convert";

async fn handler_sub(job_ids: &[String]) -> Result<(), Error> {
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
    let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));

    let executor = FileConvertExecutor {
        file_convert_job_repository: Arc::new(FileConvertJobRepository {
            db: db.clone(),
            clock,
            tsid_provider,
        }),
        s3_client: Arc::new(S3ClientImpl {
            client: aws_sdk_s3::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await),
            bucket: conf.image.convert.s3.ok_or_else(|| anyhow::anyhow!("s3 config is not found"))?.bucket,
        }),
        image_converter: Arc::new(ImageConverterImpl),
    };
    executor.execute(job_ids).await?;

    Ok(())
}

async fn handler(event: LambdaEvent<serde_json::Value>) -> Result<(), Error> {
    let (event, _context) = event.into_parts();

    let mut job_ids: Vec<String> = Vec::new();

    if let Ok(event) = serde_json::from_value::<SqsEvent>(event.clone()) {
        info!("sqs event");
        for v in event.records.into_iter().flat_map(|n| n.body).collect::<Vec<_>>() {
            info!("{:?}", v);
            let m = serde_json::from_str::<ImageConvertJobSqsMessage>(&v)?;
            for v in m.records {
                let p = Path::new(&v.s3.object.key);
                let job_id = p
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("file name is not found"))?
                    .to_string_lossy()
                    .to_string();
                job_ids.push(job_id);
            }
        }
    } else {
        info!("raw event");
        let key = event
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("key is not found"))?
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("key is not string"))?
            .to_string();
        let p = Path::new(&key);
        let job_id = p.file_name().ok_or_else(|| anyhow::anyhow!("file name is not found"))?.to_string_lossy();
        job_ids.push(job_id.to_string());
    }

    handler_sub(&job_ids).await?;

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
