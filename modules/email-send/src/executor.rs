use std::sync::Arc;

use core_cloud::aws::ses::SesSender;

use super::{EmailConfirmRequestParam, EmailSendJobBatchSqsMessage, EmailSendJobRepository, EmailSendJobType};

pub struct Executor {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub ses_sender: Arc<dyn SesSender + Send + Sync>,
}

impl Executor {
    pub async fn execute(&self, ms: &[EmailSendJobBatchSqsMessage]) -> anyhow::Result<()> {
        for m in ms.iter() {
            self.execute_one(m).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, m: &EmailSendJobBatchSqsMessage) -> anyhow::Result<()> {
        let job = self.email_send_job_repository.get_job(&m.job_id).await?;

        match job.typ {
            EmailSendJobType::EmailConfirm => {
                let param = job.param.ok_or(anyhow::anyhow!("param is not found"))?;
                let param = serde_json::from_str::<EmailConfirmRequestParam>(&param)?;
                self.execute_email_confirm(&m.job_id, m.batch_id, &param).await
            }
            _ => anyhow::bail!("invalid job type"),
        }
    }

    async fn execute_email_confirm(&self, job_id: &str, batch_id: i32, param: &EmailConfirmRequestParam) -> anyhow::Result<()> {
        let subject = "Opxs: メールアドレスの確認をお願いします";
        let body = &format!(
            "\
こんにちは、{user_name}様。

Opxs へのご登録ありがとうございます。

以下のリンクをクリックして、メールアドレスの確認を完了してください。

{email_confirm_url}

このメールに心当たりがない場合、またはご自身で opxs に登録を行っていない場合は、このメールを無視してください。

ご不明点やお困りの点がございましたら、お気軽にサポートまでお問い合わせください。

ありがとうございます。

Opxs サポートチーム",
            user_name = param.user_name,
            email_confirm_url = param.email_confirm_url,
        );

        self.email_send_job_repository
            .update_status_to_processing(job_id, batch_id, &param.to_email_address)
            .await?;

        self.ses_sender
            .send_mail_simple_text(&param.to_email_address, &param.from_email_address, subject, body)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use async_trait::async_trait;
    use chrono::Duration;
    use core_base::{clock::SystemClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
    use core_migration::postgres::PostgresMigrator;
    use core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use core_cloud::aws::sqs::SqsSender;

    use crate::EmailSendJobCreator;

    use super::*;

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
        let system_clock = Arc::new(SystemClockUtc {});
        let tsid_provider = Arc::new(TsidProviderImpl::new(SystemClockUtc, RandomBytesProviderImpl, 16));

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let email_send_job_repository = Arc::new(EmailSendJobRepository {
            db,
            system_clock,
            tsid_provider,
        });

        let send_email_sqs_sender = Arc::new(SqsSenderMock::new());
        let param = EmailConfirmRequestParam {
            user_name: "test_name".to_string(),
            to_email_address: "lyrise1984@gmail.com".to_string(),
            from_email_address: "no-reply@opxs-dev.omnius-labs.com".to_string(),
            email_confirm_url: "https://example.com".to_string(),
        };

        let job_creator = EmailSendJobCreator {
            email_send_job_repository: email_send_job_repository.clone(),
            send_email_sqs_sender,
        };
        let batches = job_creator.create_email_confirm_job(&param).await.unwrap();

        let ses_sender = Arc::new(SesSenderMock::new());
        let ms: Vec<EmailSendJobBatchSqsMessage> = batches
            .iter()
            .map(|n| EmailSendJobBatchSqsMessage {
                job_id: n.job_id.clone(),
                batch_id: n.batch_id,
            })
            .collect();

        let executor = Executor {
            email_send_job_repository,
            ses_sender: ses_sender.clone(),
        };
        executor.execute(&ms).await.unwrap();

        assert_eq!(*ses_sender.clone().to_address.borrow(), "lyrise1984@gmail.com".to_string());
        assert_eq!(*ses_sender.clone().from_address.borrow(), "no-reply@opxs-dev.omnius-labs.com".to_string());
        println!("{}", *ses_sender.clone().subject.borrow());
        println!("{}", *ses_sender.text_body.borrow());
    }

    struct SqsSenderMock {
        message_body: RefCell<String>,
    }

    unsafe impl Sync for SqsSenderMock {}
    unsafe impl Send for SqsSenderMock {}

    #[async_trait]
    impl SqsSender for SqsSenderMock {
        async fn send_message(&self, message_body: &str) -> anyhow::Result<()> {
            *self.message_body.borrow_mut() = message_body.to_string();
            Ok(())
        }
    }

    impl SqsSenderMock {
        pub fn new() -> Self {
            Self {
                message_body: RefCell::new("".to_string()),
            }
        }
    }

    struct SesSenderMock {
        to_address: RefCell<String>,
        from_address: RefCell<String>,
        subject: RefCell<String>,
        text_body: RefCell<String>,
    }

    unsafe impl Sync for SesSenderMock {}
    unsafe impl Send for SesSenderMock {}

    #[async_trait]
    impl SesSender for SesSenderMock {
        async fn send_mail_simple_text(&self, to_address: &str, from_address: &str, subject: &str, text_body: &str) -> anyhow::Result<()> {
            *self.to_address.borrow_mut() = to_address.to_string();
            *self.from_address.borrow_mut() = from_address.to_string();
            *self.subject.borrow_mut() = subject.to_string();
            *self.text_body.borrow_mut() = text_body.to_string();
            Ok(())
        }
    }

    impl SesSenderMock {
        pub fn new() -> Self {
            Self {
                to_address: RefCell::new("".to_string()),
                from_address: RefCell::new("".to_string()),
                subject: RefCell::new("".to_string()),
                text_body: RefCell::new("".to_string()),
            }
        }
    }
}
