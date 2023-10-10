use std::sync::Arc;

use omnius_core_cloud::aws::sqs::SqsSender;
use opxs_shared::message::batch::email_send::{EmailConfirmRequestParam, EmailSendJobMessage};

use super::EmailSendJobRepository;

#[allow(unused)]
pub struct EmailSendJobCreator {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
}

impl EmailSendJobCreator {
    #[allow(unused)]
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<()> {
        let id = self.email_send_job_repository.create_email_confirm_job(param).await?;
        let message = EmailSendJobMessage { id };
        self.send_email_sqs_sender.send_message(&serde_json::to_string(&message).unwrap()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use omnius_core_base::clock::SystemClockUtc;
    use omnius_core_cloud::aws::sqs::SqsSenderImpl;
    use omnius_core_migration::Migrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use crate::service::email_send::EmailSendJobRepository;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
        let migrator = Migrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let system_clock = Arc::new(SystemClockUtc {});
        let email_send_job_repository = Arc::new(EmailSendJobRepository { db, system_clock });
        let sdk_config = aws_config::from_env().load().await;
        let send_email_sqs_sender = Arc::new(SqsSenderImpl {
            client: aws_sdk_sqs::Client::new(&sdk_config),
            queue_url: "opxs-batch-email-send-sqs".to_string(),
            delay_seconds: None,
        });

        let job_creator = EmailSendJobCreator {
            email_send_job_repository,
            send_email_sqs_sender,
        };

        let param = EmailConfirmRequestParam {
            email: "lyrise1984@gmail.com".to_string(),
            user_name: "test_name".to_string(),
            email_confirm_url: "https://example.com".to_string(),
        };

        job_creator.create_email_confirm_job(&param).await.unwrap();
    }
}
