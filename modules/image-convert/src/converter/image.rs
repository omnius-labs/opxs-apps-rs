use std::path::Path;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{error, instrument};

#[async_trait]
pub trait ImageConverter {
    async fn convert(&self, source: &str, target: &str) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct ImageConverterImpl;

#[async_trait]
impl ImageConverter for ImageConverterImpl {
    #[instrument]
    async fn convert(&self, source: &str, target: &str) -> anyhow::Result<()> {
        let image_converter_dir = std::env::var("IMAGE_CONVERTER_DIR").map_err(|_| anyhow::anyhow!("IMAGE_CONVERTER is not set"))?;
        let image_converter = Path::new(&image_converter_dir).join("Omnius.ImageConverter");

        let output = Command::new(image_converter).arg(source).arg(target).output().await?;

        if !output.status.success() {
            let stdout_message = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_message = String::from_utf8_lossy(&output.stderr).to_string();

            error!(stdout = stdout_message, stderr = stderr_message, "image converter failed");

            anyhow::bail!(format!("failed to convert image: {}", stderr_message));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use crate::converter::ImageConverter;

    #[ignore]
    #[tokio::test]
    async fn simple_test() {
        env::set_var("IMAGE_CONVERTER_DIR", "/home/lyrise/bin/image-converter");
        let base_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/converter/test"));
        let input = base_path.join("test.avif").display().to_string();
        let output = base_path.join("test.png").display().to_string();

        let image_converter = crate::converter::ImageConverterImpl;

        image_converter.convert(input.as_str(), output.as_str()).await.unwrap();
    }
}
