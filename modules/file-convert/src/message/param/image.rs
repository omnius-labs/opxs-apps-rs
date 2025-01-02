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

impl FileConvertImageInputFileType {
    pub fn to_extension(&self) -> &str {
        match self {
            FileConvertImageInputFileType::Unknown => "",
            FileConvertImageInputFileType::Gif => "gif",
            FileConvertImageInputFileType::Jpg => "jpg",
            FileConvertImageInputFileType::Png => "png",
            FileConvertImageInputFileType::WebP => "webp",
            FileConvertImageInputFileType::Bmp => "bmp",
            FileConvertImageInputFileType::Heif => "heif",
            FileConvertImageInputFileType::Heic => "heic",
            FileConvertImageInputFileType::Avif => "avif",
            FileConvertImageInputFileType::Svg => "svg",
        }
    }
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
    Avif,
    Svg,
}

impl FileConvertImageOutputFileType {
    pub fn to_extension(&self) -> &str {
        match self {
            FileConvertImageOutputFileType::Gif => "gif",
            FileConvertImageOutputFileType::Jpg => "jpg",
            FileConvertImageOutputFileType::Png => "png",
            FileConvertImageOutputFileType::WebP => "webp",
            FileConvertImageOutputFileType::Bmp => "bmp",
            FileConvertImageOutputFileType::Avif => "avif",
            FileConvertImageOutputFileType::Svg => "svg",
        }
    }
}
