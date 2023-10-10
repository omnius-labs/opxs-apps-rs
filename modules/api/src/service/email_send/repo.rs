use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::clock::SystemClock;
use opxs_shared::message::batch::email_send::{EmailConfirmRequestParam, EmailSendJobStatus, EmailSendJobType};
use sqlx::PgPool;

use crate::common::AppError;

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
}

impl EmailSendJobRepository {
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> Result<i64, AppError> {
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
        .await
        .map_err(|e| AppError::UnexpectedError(e.into()))?;

        Ok(job_id)
    }
}
