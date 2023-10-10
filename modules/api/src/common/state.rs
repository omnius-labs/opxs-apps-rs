use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie;
use chrono::Duration;
use omnius_core_base::{clock::SystemClockUtc, random_bytes::RandomBytesProviderImpl};
use omnius_core_cloud::aws::sqs::SqsSenderImpl;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::{AppConfig, AppInfo, AppService};

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

        let system_clock = Arc::new(SystemClockUtc {});
        let random_bytes_provider = Arc::new(RandomBytesProviderImpl {});

        let sdk_config = aws_config::load_from_env().await;
        let send_email_sqs_sender = Arc::new(SqsSenderImpl {
            client: aws_sdk_sqs::Client::new(&sdk_config),
            queue_url: "opxs-batch-email-send-sqs".to_string(),
            delay_seconds: None,
        });

        let service = Arc::new(AppService::new(
            &info,
            &conf,
            db.clone(),
            system_clock,
            random_bytes_provider,
            send_email_sqs_sender,
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
