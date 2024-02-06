use std::path::Path;

use tokio::process::Command;
use tracing::error;

pub struct ImageConverter;

#[allow(dead_code)]
impl ImageConverter {
    pub async fn convert(input: &str, output: &str) -> anyhow::Result<()> {
        let base_path = std::env::var("IMAGE_CONVERTER_DIR").map_err(|_| anyhow::anyhow!("IMAGE_CONVERTER is not set"))?;
        let converter = Path::new(&base_path).join("ImageConverter");

        let output = Command::new(converter).arg(input).arg(output).output().await?;

        if !output.status.success() {
            let stdout_message = String::from_utf8_lossy(&output.stdout).to_string();
            error!("stdout: {}", stdout_message);

            let stderr_message = String::from_utf8_lossy(&output.stderr).to_string();
            error!("stderr: {}", stderr_message);

            anyhow::bail!(format!("failed to convert image: stdout:{} stderr:{}", stdout_message, stderr_message));
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

        ImageConverter::convert(input.as_str(), output.as_str()).await.unwrap();
    }
}
