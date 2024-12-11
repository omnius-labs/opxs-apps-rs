use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileConvertImageRequestParam {
    pub in_type: FileConvertImageInputFileType,
    pub out_type: FileConvertImageOutputFileType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileConvertImageInputFileType {
    Unknown,
    Gif,
    Jpg,
    Png,
    #[serde(rename = "webp")]
    WebP,
    Bmp,
    Heif,
    Heic,
    Avif,
    Svg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileConvertImageOutputFileType {
    Gif,
    Jpg,
    Png,
    #[serde(rename = "webp")]
    WebP,
    Bmp,
    Heif,
    Heic,
    Avif,
    Svg,
}
