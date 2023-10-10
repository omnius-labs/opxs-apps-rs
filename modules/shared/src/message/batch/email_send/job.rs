use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "email_send_job_type")]
pub enum EmailSendJobType {
    Unknown,
    EmailConfirm,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "email_send_job_status")]
pub enum EmailSendJobStatus {
    Unknown,
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJob {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub job_type: EmailSendJobType,
    #[sqlx(rename = "status")]
    pub job_status: EmailSendJobStatus,
    pub param: Option<String>,
    pub failed_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
