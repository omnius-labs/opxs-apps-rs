use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileConvertMetaRequestParam {
    pub in_type: FileConvertMetaInputFileType,
    pub out_type: FileConvertMetaOutputFileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileConvertMetaInputFileType {
    Png,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileConvertMetaOutputFileType {
    StableDiffusion,
}
