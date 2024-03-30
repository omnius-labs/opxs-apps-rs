use std::{path::Path, sync::Arc};

use chrono::{Duration, Utc};
use core_base::clock::SystemClock;
use core_cloud::aws::s3::S3Client;
use opxs_base::AppError;

use crate::{ImageConvertFile, ImageConvertJobStatus, ImageConvertRequestParam, ImageFormat};

use super::ImageConvertJobRepository;

pub struct ImageConvertJobCreator {
    pub image_convert_job_repository: Arc<ImageConvertJobRepository>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl ImageConvertJobCreator {
    pub async fn create_image_convert_job(&self, job_id: &str, filename: &str, format: &ImageFormat) -> Result<String, AppError> {
        let filename_without_extension = Path::new(filename)
            .file_stem()
            .ok_or(anyhow::anyhow!("invalid filename"))?
            .to_str()
            .ok_or(anyhow::anyhow!("invalid filename"))?;
        let origin_format = ImageFormat::from_filename(filename);

        let input = ImageConvertFile {
            filename: filename.to_string(),
            format: origin_format,
        };
        let output = ImageConvertFile {
            filename: format!("{}.{}", filename_without_extension, format.get_extension()),
            format: format.clone(),
        };

        let param = ImageConvertRequestParam {
            input: input.clone(),
            output,
        };
        self.image_convert_job_repository.create_image_convert_job(job_id, &param).await?;

        let now = self.system_clock.now();
        let expires_in = Duration::minutes(10);
        let upload_uri = self
            .s3_client
            .gen_put_presigned_uri(format!("in/{}", job_id).as_str(), now, expires_in)
            .await?;

        self.image_convert_job_repository.update_status_to_waiting(job_id).await?;

        Ok(upload_uri)
    }

    pub async fn get_status(&self, job_id: &str) -> Result<(ImageConvertJobStatus, Option<String>), AppError> {
        let job = self.image_convert_job_repository.get_job(job_id).await?;

        if job.status == ImageConvertJobStatus::Completed {
            let param = job.param.ok_or(anyhow::anyhow!("param is not found"))?;
            let param = serde_json::from_str::<ImageConvertRequestParam>(&param).map_err(|e| anyhow::anyhow!(e))?;

            let now = self.system_clock.now();
            let expires_in = Duration::minutes(10);
            let download_uri = self
                .s3_client
                .gen_get_presigned_uri(format!("out/{}", job_id).as_str(), now, expires_in, &param.output.filename)
                .await?;

            Ok((job.status, Some(download_uri)))
        } else {
            Ok((job.status, None))
        }
    }
}
