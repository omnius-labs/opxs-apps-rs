use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileConvertJobType {
    Unknown,
    Image,
    Meta,
}

impl sqlx::Type<sqlx::Postgres> for FileConvertJobType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for FileConvertJobType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            FileConvertJobType::Image => buf.extend_from_slice(b"Image"),
            FileConvertJobType::Meta => buf.extend_from_slice(b"Meta"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for FileConvertJobType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Image") => Ok(FileConvertJobType::Image),
            Ok("Meta") => Ok(FileConvertJobType::Meta),
            _ => Ok(FileConvertJobType::Unknown),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum FileConvertJobStatus {
    Unknown,
    Preparing,
    Waiting,
    Processing,
    Completed,
    Rejected,
    Failed,
}

impl sqlx::Type<sqlx::Postgres> for FileConvertJobStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for FileConvertJobStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            FileConvertJobStatus::Preparing => buf.extend_from_slice(b"Preparing"),
            FileConvertJobStatus::Waiting => buf.extend_from_slice(b"Waiting"),
            FileConvertJobStatus::Processing => buf.extend_from_slice(b"Processing"),
            FileConvertJobStatus::Completed => buf.extend_from_slice(b"Completed"),
            FileConvertJobStatus::Rejected => buf.extend_from_slice(b"Rejected"),
            FileConvertJobStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for FileConvertJobStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Preparing") => Ok(FileConvertJobStatus::Preparing),
            Ok("Waiting") => Ok(FileConvertJobStatus::Waiting),
            Ok("Processing") => Ok(FileConvertJobStatus::Processing),
            Ok("Completed") => Ok(FileConvertJobStatus::Completed),
            Ok("Rejected") => Ok(FileConvertJobStatus::Rejected),
            Ok("Failed") => Ok(FileConvertJobStatus::Failed),
            _ => Ok(FileConvertJobStatus::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct FileConvertJob {
    pub id: String,
    pub user_id: Option<String>,
    #[sqlx(rename = "type")]
    pub typ: FileConvertJobType,
    pub status: FileConvertJobStatus,
    pub param: Option<String>,
    pub in_file_name: String,
    pub out_file_name: String,
    pub failed_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
