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

impl sqlx::Decode<'_, sqlx::Postgres> for EmailSendJobType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("EmailConfirm") => Ok(EmailSendJobType::EmailConfirm),
            _ => Ok(EmailSendJobType::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJob {
    pub id: String,
    pub batch_count: i32,
    pub email_address_count: i32,
    #[sqlx(rename = "type")]
    pub typ: EmailSendJobType,
    pub param: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendJobBatchStatus {
    Unknown,
    Waiting,
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
            EmailSendJobBatchStatus::Waiting => buf.extend_from_slice(b"Waiting"),
            EmailSendJobBatchStatus::Processing => buf.extend_from_slice(b"Processing"),
            EmailSendJobBatchStatus::Completed => buf.extend_from_slice(b"Completed"),
            EmailSendJobBatchStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for EmailSendJobBatchStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Waiting") => Ok(EmailSendJobBatchStatus::Waiting),
            Ok("Processing") => Ok(EmailSendJobBatchStatus::Processing),
            Ok("Completed") => Ok(EmailSendJobBatchStatus::Completed),
            Ok("Failed") => Ok(EmailSendJobBatchStatus::Failed),
            _ => Ok(EmailSendJobBatchStatus::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJobBatch {
    pub job_id: String,
    pub batch_id: i32,
    pub status: EmailSendJobBatchStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendJobBatchDetailStatus {
    Unknown,
    Waiting,
    Processing,
    Completed,
    Rejected,
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
            EmailSendJobBatchDetailStatus::Waiting => buf.extend_from_slice(b"Waiting"),
            EmailSendJobBatchDetailStatus::Processing => buf.extend_from_slice(b"Processing"),
            EmailSendJobBatchDetailStatus::Completed => buf.extend_from_slice(b"Completed"),
            EmailSendJobBatchDetailStatus::Rejected => buf.extend_from_slice(b"Rejected"),
            EmailSendJobBatchDetailStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for EmailSendJobBatchDetailStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Waiting") => Ok(EmailSendJobBatchDetailStatus::Waiting),
            Ok("Processing") => Ok(EmailSendJobBatchDetailStatus::Processing),
            Ok("Completed") => Ok(EmailSendJobBatchDetailStatus::Completed),
            Ok("Rejected") => Ok(EmailSendJobBatchDetailStatus::Rejected),
            Ok("Failed") => Ok(EmailSendJobBatchDetailStatus::Failed),
            _ => Ok(EmailSendJobBatchDetailStatus::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct EmailSendJobBatchDetail {
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
