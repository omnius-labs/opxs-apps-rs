use std::{path::Path, process::Stdio};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as BASE64};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::{FileConvertImageInputFileType, FileConvertImageOutputFileType, prelude::*};

#[async_trait]
pub trait ImageConverter {
    async fn convert(
        &self,
        in_path: &Path,
        in_type: &FileConvertImageInputFileType,
        out_path: &Path,
        out_type: &FileConvertImageOutputFileType,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct ImageConverterImpl;

#[async_trait]
impl ImageConverter for ImageConverterImpl {
    async fn convert(
        &self,
        in_path: &Path,
        in_type: &FileConvertImageInputFileType,
        out_path: &Path,
        out_type: &FileConvertImageOutputFileType,
    ) -> Result<()> {
        let image_converter_dir = std::env::var("IMAGE_CONVERTER_DIR")?;
        let image_converter = Path::new(&image_converter_dir).join("Omnius.ImageConverter");

        let image_converter_option = ImageConverterOption {
            in_path: in_path.to_string_lossy().to_string(),
            in_type: in_type.clone(),
            out_path: out_path.to_string_lossy().to_string(),
            out_type: out_type.clone(),
        };
        let image_converter_option = BASE64.encode(serde_json::to_string(&image_converter_option)?);

        let mut cmd = Command::new(image_converter)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = cmd
            .stdin
            .take()
            .ok_or_else(|| Error::builder().kind(ErrorKind::UnexpectedError).build())?;
        stdin.write_all(image_converter_option.as_bytes()).await?;
        stdin.write_all(b"\n").await?;

        let mut stdout = cmd
            .stdout
            .take()
            .ok_or_else(|| Error::builder().kind(ErrorKind::UnexpectedError).build())?;
        let stdout_handle = tokio::spawn(async move {
            let mut v = String::new();
            stdout.read_to_string(&mut v).await.ok();
            v
        });

        let mut stderr = cmd
            .stderr
            .take()
            .ok_or_else(|| Error::builder().kind(ErrorKind::UnexpectedError).build())?;
        let stderr_handle = tokio::spawn(async move {
            let mut v = String::new();
            stderr.read_to_string(&mut v).await.ok();
            v
        });

        let (stdout_result, stderr_result) = tokio::try_join!(stdout_handle, stderr_handle)?;

        let status = cmd.wait().await?;
        if !status.success() {
            return Err(Error::builder()
                .kind(ErrorKind::ProcessFailed)
                .message(format!("Process failed.\nstdout: {}\nstderr: {}", stdout_result, stderr_result))
                .build());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ImageConverterOption {
    pub in_path: String,
    pub in_type: FileConvertImageInputFileType,
    pub out_path: String,
    pub out_type: FileConvertImageOutputFileType,
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use crate::{FileConvertImageInputFileType, FileConvertImageOutputFileType, ImageConverter as _, ImageConverterImpl};

    #[ignore]
    #[tokio::test]
    async fn simple_test() {
        unsafe {
            env::set_var("IMAGE_CONVERTER_DIR", "/home/lyrise/repos/omnius-labs/image-converter-cs/pub/linux-x64");
        }
        let base_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/converter/test"));
        let input = base_path.join("test.avif");
        let output = base_path.join("test.png");

        let converter = ImageConverterImpl;

        converter
            .convert(
                &input,
                &FileConvertImageInputFileType::Avif,
                &output,
                &FileConvertImageOutputFileType::Png,
            )
            .await
            .unwrap();
    }
}
