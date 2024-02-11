use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum ImageConvertJobStatus {
    Unknown,
    Preparing,
    Waiting,
    Processing,
    Completed,
    Rejected,
    Failed,
}

impl sqlx::Type<sqlx::Postgres> for ImageConvertJobStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ImageConvertJobStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            ImageConvertJobStatus::Preparing => buf.extend_from_slice(b"Preparing"),
            ImageConvertJobStatus::Waiting => buf.extend_from_slice(b"Waiting"),
            ImageConvertJobStatus::Processing => buf.extend_from_slice(b"Processing"),
            ImageConvertJobStatus::Completed => buf.extend_from_slice(b"Completed"),
            ImageConvertJobStatus::Rejected => buf.extend_from_slice(b"Rejected"),
            ImageConvertJobStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ImageConvertJobStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Preparing") => Ok(ImageConvertJobStatus::Preparing),
            Ok("Waiting") => Ok(ImageConvertJobStatus::Waiting),
            Ok("Processing") => Ok(ImageConvertJobStatus::Processing),
            Ok("Completed") => Ok(ImageConvertJobStatus::Completed),
            Ok("Rejected") => Ok(ImageConvertJobStatus::Rejected),
            Ok("Failed") => Ok(ImageConvertJobStatus::Failed),
            _ => Ok(ImageConvertJobStatus::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct ImageConvertJob {
    pub id: String,
    pub param: Option<String>,
    pub status: ImageConvertJobStatus,
    pub failed_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
