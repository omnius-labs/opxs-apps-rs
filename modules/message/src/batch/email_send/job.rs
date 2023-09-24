use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "email_send_job_type")]
pub enum EmailSendJobType {
    EmailConfirm,
}

pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(sqlx::FromRow)]
pub struct EmailSendJob {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub job_type: EmailSendJobType,
    #[sqlx(rename = "status")]
    pub job_status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
