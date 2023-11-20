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
        println!("---- 1 ----");
        for m in ms.iter() {
            self.execute_one(m).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, m: &EmailSendJobSqsMessage) -> anyhow::Result<()> {
        let job = self.email_send_job_repository.get_job(&m.id).await?;
        println!("---- 2 ----");

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
        let subject = "[opxs] メールアドレスの確認をお願いします";
        let body = &format!(
            "\
こんにちは、{user_name}様。

opxs へのご登録ありがとうございます。

以下のリンクをクリックして、メールアドレスの確認を完了してください。

{email_confirm_url}

このメールに心当たりがない場合、またはご自身で opxs に登録を行っていない場合は、このメールを無視してください。

ご不明点やお困りの点がございましたら、お気軽にサポートまでお問い合わせください。

ありがとうございます。

opxs サポートチーム",
            user_name = param.user_name,
            email_confirm_url = param.email_confirm_url,
        );

        println!("---- 3 ----");

        self.ses_sender
            .send_mail_simple_text(&param.email, &self.from_address, subject, body)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use core_cloud::aws::ses::SesSenderImpl;
    use core_migration::Migrator;
    use core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use crate::EmailSendJobStatus;

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

        // テスト用のデータを作成
        let param = EmailConfirmRequestParam {
            email: "lyrise1984@gmail.com".to_string(),
            user_name: "Lyrise".to_string(),
            email_confirm_url: "http://example.com".to_string(),
        };
        let (job_id,): (i64,) = sqlx::query_as(
            r#"
INSERT INTO email_send_jobs (type, status, param)
    VALUES ($1, $2, $3)
    RETURNING id;
"#,
        )
        .bind(EmailSendJobType::EmailConfirm)
        .bind(EmailSendJobStatus::Pending)
        .bind(&serde_json::to_string(&param).unwrap())
        .fetch_one(db.as_ref())
        .await
        .unwrap();

        let executor = Executor {
            email_send_job_repository: Arc::new(EmailSendJobRepository { db: db.clone() }),
            ses_sender: Arc::new(SesSenderImpl {
                client: aws_sdk_sesv2::Client::new(&aws_config::load_from_env().await),
                configuration_set_name: Some("opxs-email-send".to_string()),
            }),
            from_address: "no-reply@opxs-dev.omnius-labs.com".to_string(),
        };

        let ms = vec![EmailSendJobSqsMessage { id: job_id }];

        executor.execute(&ms).await.unwrap();
    }
}
