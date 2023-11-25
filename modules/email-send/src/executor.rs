use std::sync::Arc;

use core_cloud::aws::ses::SesSender;

use super::{EmailConfirmRequestParam, EmailSendJobRepository, EmailSendJobSqsMessage, EmailSendJobType};

pub struct Executor {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub ses_sender: Arc<dyn SesSender + Send + Sync>,
    pub from_address: String,
}

impl Executor {
    pub async fn execute(&self, ms: &[EmailSendJobSqsMessage]) -> anyhow::Result<()> {
        for m in ms.iter() {
            self.execute_one(m).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, m: &EmailSendJobSqsMessage) -> anyhow::Result<()> {
        let job = self.email_send_job_repository.get_job(&m.id).await?;

        match job.job_type {
            EmailSendJobType::EmailConfirm => {
                let param = job.param.ok_or(anyhow::anyhow!("param is not found"))?;
                let param = serde_json::from_str::<EmailConfirmRequestParam>(&param)?;
                self.execute_email_confirm(&param).await
            }
            _ => anyhow::bail!("invalid job type"),
        }
    }

    async fn execute_email_confirm(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<()> {
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

        self.ses_sender
            .send_mail_simple_text(&param.email, &self.from_address, subject, body)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use async_trait::async_trait;
    use chrono::Duration;
    use core_migration::Migrator;
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

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = Migrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let email_send_job_repository = Arc::new(EmailSendJobRepository { db });

        let send_email_sqs_sender = Arc::new(SqsSenderMock::new());
        let param = EmailConfirmRequestParam {
            email: "lyrise1984@gmail.com".to_string(),
            user_name: "test_name".to_string(),
            email_confirm_url: "https://example.com".to_string(),
        };

        let job_creator = EmailSendJobCreator {
            email_send_job_repository: email_send_job_repository.clone(),
            send_email_sqs_sender,
        };
        let job_id = job_creator.create_email_confirm_job(&param).await.unwrap();

        let ses_sender = Arc::new(SesSenderMock::new());
        let ms = vec![EmailSendJobSqsMessage { id: job_id }];

        let executor = Executor {
            email_send_job_repository,
            ses_sender: ses_sender.clone(),
            from_address: "no-reply@opxs-dev.omnius-labs.com".to_string(),
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
