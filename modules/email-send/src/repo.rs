use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::clock::Clock;
use sqlx::PgPool;

use crate::EmailSendJobBatchDetail;

use super::{EmailConfirmRequestParam, EmailSendJob, EmailSendJobBatch, EmailSendJobBatchDetailStatus, EmailSendJobBatchStatus, EmailSendJobType};

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
}

impl EmailSendJobRepository {
    pub async fn create_job(&self, job_id: &str, param: &EmailConfirmRequestParam) -> anyhow::Result<()> {
        let now = self.clock.now();

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
        .bind(&serde_json::to_string(param)?)
        .bind(now)
        .execute(&mut *tx)
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
        .execute(&mut *tx)
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
        .execute(&mut *tx)
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

    pub async fn set_message_id(&self, job_id: &str, batch_id: i32, email_address: &str, message_id: &str) -> anyhow::Result<()> {
        let now = self.clock.now();

        sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET message_id = $5, updated_at = $4
    WHERE job_id = $1 AND batch_id = $2 AND email_address = $3
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .bind(email_address)
        .bind(now)
        .bind(message_id)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn update_status_to_waiting(&self, job_id: &str) -> anyhow::Result<()> {
        self.update_status_by_job_id(job_id, &EmailSendJobBatchDetailStatus::Preparing, &EmailSendJobBatchDetailStatus::Waiting)
            .await
    }

    pub async fn update_status_to_processing(&self, job_id: &str, batch_id: i32, email_address: &str) -> anyhow::Result<()> {
        self.update_status_by_email_address(
            job_id,
            batch_id,
            email_address,
            &EmailSendJobBatchDetailStatus::Waiting,
            &EmailSendJobBatchDetailStatus::Processing,
        )
        .await
    }

    pub async fn update_status_to_requested(&self, message_id: &str) -> anyhow::Result<()> {
        self.update_status_by_message_id(
            message_id,
            &EmailSendJobBatchDetailStatus::Processing,
            &EmailSendJobBatchDetailStatus::Requested,
        )
        .await
    }

    async fn update_status_by_job_id(
        &self,
        job_id: &str,
        old: &EmailSendJobBatchDetailStatus,
        new: &EmailSendJobBatchDetailStatus,
    ) -> anyhow::Result<()> {
        let mut tx = self.db.begin().await?;
        let now = self.clock.now();

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batches
    SET status = $4, updated_at = $2
    WHERE job_id = $1 AND status = $3
"#,
        )
        .bind(job_id)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = $4, updated_at = $2
    WHERE job_id = $1 AND status = $3
"#,
        )
        .bind(job_id)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        tx.commit().await?;

        Ok(())
    }

    async fn update_status_by_email_address(
        &self,
        job_id: &str,
        batch_id: i32,
        email_address: &str,
        old: &EmailSendJobBatchDetailStatus,
        new: &EmailSendJobBatchDetailStatus,
    ) -> anyhow::Result<()> {
        let mut tx = self.db.begin().await?;
        let now = self.clock.now();

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = $6, updated_at = $4
    WHERE job_id = $1 AND batch_id = $2 AND email_address = $3 AND status = $5
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .bind(email_address)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        sqlx::query(
            r#"
UPDATE email_send_job_batches
    SET status = (
        SELECT CASE WHEN COUNT(1) > 0 THEN $4 ELSE $5 END
            FROM email_send_job_batch_details
            WHERE job_id = $1 AND batch_id = $2 AND status = $4
    ), updated_at = $3
    WHERE job_id = $1 AND batch_id = $2 AND status = $4
"#,
        )
        .bind(job_id)
        .bind(batch_id)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_status_by_message_id(
        &self,
        message_id: &str,
        old: &EmailSendJobBatchDetailStatus,
        new: &EmailSendJobBatchDetailStatus,
    ) -> anyhow::Result<()> {
        let mut tx = self.db.begin().await?;
        let now = self.clock.now();

        let res = sqlx::query(
            r#"
UPDATE email_send_job_batch_details
    SET status = $4, updated_at = $2
    WHERE message_id = $1 AND status = $3
"#,
        )
        .bind(message_id)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        if res.rows_affected() < 1 {
            anyhow::bail!("no rows affected");
        }

        let (job_id, batch_id): (String, i32) = sqlx::query_as(
            r#"
SELECT job_id, batch_id
    FROM email_send_job_batch_details
    WHERE message_id = $1
"#,
        )
        .bind(message_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
UPDATE email_send_job_batches
    SET status = (
        SELECT CASE WHEN COUNT(1) > 0 THEN $4 ELSE $5 END
            FROM email_send_job_batch_details
            WHERE job_id = $1 AND batch_id = $2 AND status = $4
    ), updated_at = $3
    WHERE job_id = $1 AND batch_id = $2 AND status = $4
"#,
        )
        .bind(job_id.as_str())
        .bind(batch_id)
        .bind(now)
        .bind(old)
        .bind(new)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
