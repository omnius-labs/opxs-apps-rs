use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie;
use chrono::Duration;
use sqlx::{postgres::PgPoolOptions, PgPool};

use core_base::{clock::RealClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
use core_cloud::aws::{s3::S3ClientImpl, sqs::SqsSenderImpl};

use opxs_base::{AppConfig, AppInfo};

use super::service::AppService;

#[derive(Clone)]
pub struct AppState {
    pub info: AppInfo,
    pub conf: AppConfig,
    pub db: Arc<PgPool>,
    pub service: Arc<AppService>,
    cookie_key: cookie::Key,
}

impl AppState {
    pub async fn new(info: AppInfo, conf: AppConfig) -> anyhow::Result<Self> {
        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&conf.postgres.url)
                .await?,
        );

        let clock = Arc::new(RealClockUtc);
        let random_bytes_provider = Arc::new(RandomBytesProviderImpl);
        let tsid_provider = Arc::new(TsidProviderImpl::new(RealClockUtc, RandomBytesProviderImpl, 16));

        let sdk_config = aws_config::load_from_env().await;
        let send_email_sqs_sender = Arc::new(SqsSenderImpl {
            client: aws_sdk_sqs::Client::new(&sdk_config),
            queue_url: "opxs-batch-email-send-sqs".to_string(),
            delay_seconds: None,
        });
        let image_convert_s3_client = Arc::new(S3ClientImpl {
            client: aws_sdk_s3::Client::new(&aws_config::load_from_env().await),
            bucket: conf.image_convert.s3.bucket.clone(),
        });

        let service = Arc::new(AppService::new(
            &info,
            &conf,
            db.clone(),
            clock,
            random_bytes_provider,
            tsid_provider,
            send_email_sqs_sender,
            image_convert_s3_client,
        ));

        Ok(Self {
            info,
            conf,
            db,
            service,
            cookie_key: cookie::Key::generate(),
        })
    }
}

impl FromRef<AppState> for cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
