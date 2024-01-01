use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendJobType {
    Unknown,
    EmailConfirm,
}

impl sqlx::Type<sqlx::Postgres> for EmailSendJobType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for EmailSendJobType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            EmailSendJobType::EmailConfirm => buf.extend_from_slice(b"EmailConfirm"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJob {
    pub job_id: String,
    pub batch_count: u32,
    pub email_address_count: u32,
    #[sqlx(rename = "type")]
    pub job_type: EmailSendJobType,
    pub param: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendJobBatchStatus {
    Unknown,
    Pending,
    Processing,
    Completed,
    Failed,
}

impl sqlx::Type<sqlx::Postgres> for EmailSendJobBatchStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for EmailSendJobBatchStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            EmailSendJobBatchStatus::Pending => buf.extend_from_slice(b"Pending"),
            EmailSendJobBatchStatus::Processing => buf.extend_from_slice(b"Processing"),
            EmailSendJobBatchStatus::Completed => buf.extend_from_slice(b"Completed"),
            EmailSendJobBatchStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJobBatches {
    pub job_id: String,
    pub batch_id: i32,
    pub status: EmailSendJobBatchStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendJobBatchDetailStatus {
    Unknown,
    Pending,
    Processing,
    Completed,
    Failed,
}

impl sqlx::Type<sqlx::Postgres> for EmailSendJobBatchDetailStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for EmailSendJobBatchDetailStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            EmailSendJobBatchDetailStatus::Pending => buf.extend_from_slice(b"Pending"),
            EmailSendJobBatchDetailStatus::Processing => buf.extend_from_slice(b"Processing"),
            EmailSendJobBatchDetailStatus::Completed => buf.extend_from_slice(b"Completed"),
            EmailSendJobBatchDetailStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJobBatchDetails {
    pub job_id: String,
    pub batch_id: i32,
    pub email_address: String,
    pub retry_count: i32,
    pub message_id: String,
    pub status: EmailSendJobBatchDetailStatus,
    pub failed_reason: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
