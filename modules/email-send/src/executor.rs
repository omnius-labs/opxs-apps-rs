use std::sync::Arc;

use omnius_core_cloud::aws::ses::SesSender;

use super::{
    EmailConfirmRequestParam, EmailSendJobBatchSqsMessage, EmailSendJobRepository, EmailSendJobType,
};

pub struct EmailSendExecutor {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub ses_sender: Arc<dyn SesSender + Send + Sync>,
}

impl EmailSendExecutor {
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
                let param = job
                    .param
                    .ok_or_else(|| anyhow::anyhow!("param is not found"))?;
                let param = serde_json::from_str::<EmailConfirmRequestParam>(&param)?;
                self.execute_email_confirm(&m.job_id, m.batch_id, &param)
                    .await
            }
            _ => anyhow::bail!("invalid job type"),
        }
    }

    async fn execute_email_confirm(
        &self,
        job_id: &str,
        batch_id: i32,
        param: &EmailConfirmRequestParam,
    ) -> anyhow::Result<()> {
        self.email_send_job_repository
            .update_status_to_processing(job_id, batch_id, &param.to_email_address)
            .await?;

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

        let message_id = self
            .ses_sender
            .send_mail_simple_text(
                &param.to_email_address,
                &param.from_email_address,
                subject,
                body,
            )
            .await?;

        self.email_send_job_repository
            .set_message_id(
                job_id,
                batch_id,
                &param.to_email_address,
                message_id.as_str(),
            )
            .await?;

        self.email_send_job_repository
            .update_status_to_requested(message_id.as_str())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use parking_lot::Mutex;
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_base::{
        clock::ClockUtc,
        random_bytes::RandomBytesProviderImpl,
        tsid::{TsidProvider, TsidProviderImpl},
    };
    use omnius_core_cloud::aws::{ses::SesSenderMock, sqs::SqsSenderMock};
    use omnius_core_migration::postgres::PostgresMigrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::{shared, EmailSendJobCreator};

    use super::*;

    #[tokio::test]
    async fn simple_test() -> TestResult {
        let container = PostgresContainer::new(shared::POSTGRES_VERSION).await?;

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );
        let clock = Arc::new(ClockUtc {});
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(
            ClockUtc,
            RandomBytesProviderImpl::new(),
            16,
        )));

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(
            &container.connection_string,
            migrations_path,
            "opxs-api",
            "",
        )
        .await
        .unwrap();
        migrator.migrate().await.unwrap();

        let email_send_job_repository = Arc::new(EmailSendJobRepository { db, clock });

        let send_email_sqs_sender = Arc::new(SqsSenderMock::new());

        let job_id = tsid_provider.lock().gen().to_string();
        let job_creator = EmailSendJobCreator {
            email_send_job_repository: email_send_job_repository.clone(),
            sqs_sender: send_email_sqs_sender.clone(),
        };
        job_creator
            .create_job(
                &job_id,
                "test_name",
                "lyrise1984@gmail.com",
                "no-reply@opxs-dev.omnius-labs.com",
                "https://example.com",
            )
            .await
            .unwrap();

        let ses_sender = Arc::new(SesSenderMock::new());
        let sqs_send_message_input = send_email_sqs_sender
            .send_message_inputs
            .lock()
            .first()
            .cloned()
            .unwrap();
        let sqs_message =
            serde_json::from_str::<EmailSendJobBatchSqsMessage>(sqs_send_message_input.as_str())
                .unwrap();

        let executor = EmailSendExecutor {
            email_send_job_repository,
            ses_sender: ses_sender.clone(),
        };
        executor.execute(&[sqs_message]).await.unwrap();

        let ses_send_mail_simple_text_input = ses_sender
            .send_mail_simple_text_inputs
            .lock()
            .first()
            .cloned()
            .unwrap();

        assert_eq!(
            ses_send_mail_simple_text_input.to_address,
            "lyrise1984@gmail.com".to_string()
        );
        assert_eq!(
            ses_send_mail_simple_text_input.from_address,
            "no-reply@opxs-dev.omnius-labs.com".to_string()
        );
        println!("{}", ses_send_mail_simple_text_input.subject);
        println!("{}", ses_send_mail_simple_text_input.text_body);

        Ok(())
    }
}
