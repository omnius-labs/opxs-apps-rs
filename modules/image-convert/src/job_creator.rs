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
    pub async fn create_image_convert_job(&self, job_id: &str, original_filename: &str, out_format: &ImageFormat) -> Result<String, AppError> {
        let original_filename_without_extension = Path::new(original_filename)
            .file_stem()
            .ok_or(anyhow::anyhow!("invalid filename"))?
            .to_str()
            .ok_or(anyhow::anyhow!("invalid filename"))?;
        let in_format = ImageFormat::from_filename(original_filename);

        let input = ImageConvertFile {
            s3_key: format!("in/{}", job_id),
            filename: original_filename.to_string(),
            format: in_format,
        };
        let output = ImageConvertFile {
            s3_key: format!("out/{}", job_id),
            filename: format!("{}.{}", original_filename_without_extension, out_format.get_extension()),
            format: out_format.clone(),
        };

        let param = ImageConvertRequestParam {
            input: input.clone(),
            output,
        };
        self.image_convert_job_repository.create_image_convert_job(job_id, &param).await?;

        let now = self.system_clock.now();
        let expires_in = Duration::minutes(10);
        let upload_uri = self.s3_client.gen_put_presigned_uri(&input.s3_key, now, expires_in).await?;

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
                .gen_get_presigned_uri(&param.output.s3_key, now, expires_in, &param.output.filename)
                .await?;

            Ok((job.status, Some(download_uri)))
        } else {
            Ok((job.status, None))
        }
    }
}
