use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum ConvertFileStatus {
    Unknown,
    Preparing,
    Waiting,
    Processing,
    Completed,
    Rejected,
    Failed,
}

impl sqlx::Type<sqlx::Postgres> for ConvertFileStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ConvertFileStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        match self {
            ConvertFileStatus::Preparing => buf.extend_from_slice(b"Preparing"),
            ConvertFileStatus::Waiting => buf.extend_from_slice(b"Waiting"),
            ConvertFileStatus::Processing => buf.extend_from_slice(b"Processing"),
            ConvertFileStatus::Completed => buf.extend_from_slice(b"Completed"),
            ConvertFileStatus::Rejected => buf.extend_from_slice(b"Rejected"),
            ConvertFileStatus::Failed => buf.extend_from_slice(b"Failed"),
            _ => buf.extend_from_slice(b"Unknown"),
        }
        sqlx::encode::IsNull::No
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ConvertFileStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match value.as_str() {
            Ok("Preparing") => Ok(ConvertFileStatus::Preparing),
            Ok("Waiting") => Ok(ConvertFileStatus::Waiting),
            Ok("Processing") => Ok(ConvertFileStatus::Processing),
            Ok("Completed") => Ok(ConvertFileStatus::Completed),
            Ok("Rejected") => Ok(ConvertFileStatus::Rejected),
            Ok("Failed") => Ok(ConvertFileStatus::Failed),
            _ => Ok(ConvertFileStatus::Unknown),
        }
    }
}
