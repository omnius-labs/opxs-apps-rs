use std::sync::Arc;

use core_base::tsid::TsidProvider;
use sqlx::PgPool;

use crate::EmailSendJobBatchDetail;

use super::{EmailConfirmRequestParam, EmailSendJob, EmailSendJobBatch, EmailSendJobBatchDetailStatus, EmailSendJobBatchStatus, EmailSendJobType};

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
    pub tsid_provider: Arc<dyn TsidProvider + Send + Sync>,
}

impl EmailSendJobRepository {
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<String> {
        let job_id = self.tsid_provider.gen().to_string();

        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
        INSERT INTO email_send_jobs (id, batch_count, email_address_count, type, param)
            VALUES ($1, $2, $3, $4, $5);
        "#,
        )
        .bind(job_id.as_str())
        .bind(1)
        .bind(1)
        .bind(EmailSendJobType::EmailConfirm)
        .bind(&serde_json::to_string(param).unwrap())
        .execute(&mut tx)
        .await?;

        sqlx::query(
            r#"
        INSERT INTO email_send_job_batches (job_id, batch_id, status)
            VALUES ($1, $2, $3);
        "#,
        )
        .bind(job_id.as_str())
        .bind(0)
        .bind(EmailSendJobBatchStatus::Waiting)
        .execute(&mut tx)
        .await?;

        sqlx::query(
            r#"
        INSERT INTO email_send_job_batch_details (job_id, batch_id, email_address, retry_count, status)
            VALUES ($1, $2, $3, $4, $5);
        "#,
        )
        .bind(job_id.as_str())
        .bind(0)
        .bind(param.to_email_address.as_str())
        .bind(0)
        .bind(EmailSendJobBatchDetailStatus::Waiting)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(job_id)
    }

    pub async fn get_job(&self, id: &str) -> anyhow::Result<EmailSendJob> {
        let res: EmailSendJob = sqlx::query_as(
            r#"
SELECT *
    FROM email_send_jobs
    WHERE id = $1
"#,
        )
        .bind(id)
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(res)
    }

    pub async fn get_job_batches(&self, job_id: &str) -> anyhow::Result<Vec<EmailSendJobBatch>> {
        let res: Vec<EmailSendJobBatch> = sqlx::query_as(
            r#"
SELECT *
    FROM email_send_job_batches
    WHERE job_id = $1
"#,
        )
        .bind(job_id)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(res)
    }

    pub async fn get_job_batch_details(&self, job_id: &str, batch_id: i32) -> anyhow::Result<Vec<EmailSendJobBatchDetail>> {
        let res: Vec<EmailSendJobBatchDetail> = sqlx::query_as(
            r#"
SELECT *
    FROM email_send_job_batch_details
    WHERE job_id = $1 AND batch_id = $2
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(res)
    }

    pub async fn update_batch_job(&self, job_id: &str, batch_id: i32, status: EmailSendJobBatchStatus) -> anyhow::Result<()> {
        sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = $3
    WHERE job_id = $1 AND batch_id = $2
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .bind(status)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }
}
