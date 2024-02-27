use std::sync::Arc;

use chrono::Utc;
use core_base::clock::SystemClock;
use sqlx::PgPool;

use crate::EmailSendJobBatchDetail;

use super::{EmailConfirmRequestParam, EmailSendJob, EmailSendJobBatch, EmailSendJobBatchDetailStatus, EmailSendJobBatchStatus, EmailSendJobType};

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
}

impl EmailSendJobRepository {
    pub async fn create_email_confirm_job(&self, job_id: &str, param: &EmailConfirmRequestParam) -> anyhow::Result<()> {
        let now = self.system_clock.now();

        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
INSERT INTO email_send_jobs (id, batch_count, email_address_count, type, param, created_at)
    VALUES ($1, $2, $3, $4, $5, $6);
        "#,
        )
        .bind(job_id)
        .bind(1)
        .bind(1)
        .bind(EmailSendJobType::EmailConfirm)
        .bind(&serde_json::to_string(param).unwrap())
        .bind(now)
        .execute(&mut tx)
        .await?;

        sqlx::query(
            r#"
INSERT INTO email_send_job_batches (job_id, batch_id, status, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5);
        "#,
        )
        .bind(job_id)
        .bind(0)
        .bind(EmailSendJobBatchStatus::Preparing)
        .bind(now)
        .bind(now)
        .execute(&mut tx)
        .await?;

        sqlx::query(
            r#"
INSERT INTO email_send_job_batch_details (job_id, batch_id, email_address, retry_count, status, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
        )
        .bind(job_id)
        .bind(0)
        .bind(param.to_email_address.as_str())
        .bind(0)
        .bind(EmailSendJobBatchDetailStatus::Preparing)
        .bind(now)
        .bind(now)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
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

    pub async fn update_status_to_waiting(&self, job_id: &str) -> anyhow::Result<()> {
        let mut tx = self.db.begin().await?;
        let now = self.system_clock.now();

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batches
    SET status = 'Waiting', updated_at = $2
    WHERE job_id = $1 AND status = 'Preparing'
"#,
        )
        .bind(job_id)
        .bind(now)
        .execute(&mut tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = 'Waiting', updated_at = $2
    WHERE job_id = $1 AND status = 'Preparing'
"#,
        )
        .bind(job_id)
        .bind(now)
        .execute(&mut tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn update_status_to_processing(&self, job_id: &str, batch_id: i32, email_address: &str) -> anyhow::Result<()> {
        let mut tx = self.db.begin().await?;
        let now = self.system_clock.now();

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = 'Processing', updated_at = $3
    WHERE job_id = $1 AND email_address = $2 AND status = 'Waiting'
"#,
        )
        .bind(job_id)
        .bind(email_address)
        .bind(now)
        .execute(&mut tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        sqlx::query(
            r#"
UPDATE email_send_job_batches
    SET status = (
        SELECT CASE WHEN COUNT(1) > 0 THEN 'Waiting' ELSE 'Processing' END
            FROM email_send_job_batch_details
            WHERE job_id = $1 AND batch_id = $2 AND status = 'Waiting'
    ), updated_at = $3
    WHERE job_id = $1 AND batch_id = $2 AND status = 'Waiting'
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .bind(now)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
