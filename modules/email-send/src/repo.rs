use std::sync::Arc;

use sqlx::PgPool;

use super::{EmailConfirmRequestParam, EmailSendJob, EmailSendJobStatus, EmailSendJobType};

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
}

impl EmailSendJobRepository {
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<i64> {
        let (job_id,): (i64,) = sqlx::query_as(
            r#"
INSERT INTO email_send_jobs (type, status, param)
    VALUES ($1, $2, $3)
    RETURNING id;
"#,
        )
        .bind(EmailSendJobType::EmailConfirm)
        .bind(EmailSendJobStatus::Pending)
        .bind(&serde_json::to_string(param).unwrap())
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(job_id)
    }

    pub async fn get_job(&self, id: &i64) -> anyhow::Result<EmailSendJob> {
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
}
