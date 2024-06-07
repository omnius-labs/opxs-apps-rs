use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageConvertRequestParam {
    pub input: ImageConvertFile,
    pub output: ImageConvertFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageConvertFile {
    pub file_name: String,
    #[serde(alias = "type")]
    pub typ: ImageType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    #[default]
    Unknown,
    Gif,
    Jpg,
    Png,
    WebP,
    Bmp,
    Heif,
    Heic,
    Avif,
    Svg,
}

impl ImageType {
    pub fn from_file_name(path: &str) -> Self {
        let v = path.rsplitn(2, '.').collect::<Vec<&str>>();
        let ext = *v.first().unwrap_or(&"");
        Self::from_extension(ext)
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "gif" => ImageType::Gif,
            "jpg" => ImageType::Jpg,
            "png" => ImageType::Png,
            "webp" => ImageType::WebP,
            "heif" => ImageType::Heif,
            "heic" => ImageType::Heic,
            "avif" => ImageType::Avif,
            "svg" => ImageType::Svg,
            _ => ImageType::Unknown,
        }
    }

    pub fn get_extension(&self) -> String {
        match self {
            ImageType::Gif => "gif".to_string(),
            ImageType::Jpg => "jpg".to_string(),
            ImageType::Png => "png".to_string(),
            ImageType::WebP => "webp".to_string(),
            ImageType::Heif => "heif".to_string(),
            ImageType::Heic => "heic".to_string(),
            ImageType::Avif => "avif".to_string(),
            ImageType::Svg => "svg".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
