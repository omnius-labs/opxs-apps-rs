use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageConvertRequestParam {
    pub input: ImageConvertFile,
    pub output: ImageConvertFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageConvertFile {
    pub filename: String,
    pub format: ImageFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
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

impl ImageFormat {
    pub fn from_filename(path: &str) -> Self {
        let v = path.rsplitn(2, '.').collect::<Vec<&str>>();
        let ext = *v.first().unwrap_or(&"");
        Self::from_extension(ext)
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "gif" => ImageFormat::Gif,
            "jpg" => ImageFormat::Jpg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::WebP,
            "heif" => ImageFormat::Heif,
            "heic" => ImageFormat::Heic,
            "avif" => ImageFormat::Avif,
            "svg" => ImageFormat::Svg,
            _ => ImageFormat::Unknown,
        }
    }

    pub fn get_extension(&self) -> String {
        match self {
            ImageFormat::Gif => "gif".to_string(),
            ImageFormat::Jpg => "jpg".to_string(),
            ImageFormat::Png => "png".to_string(),
            ImageFormat::WebP => "webp".to_string(),
            ImageFormat::Heif => "heif".to_string(),
            ImageFormat::Heic => "heic".to_string(),
            ImageFormat::Avif => "avif".to_string(),
            ImageFormat::Svg => "svg".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
