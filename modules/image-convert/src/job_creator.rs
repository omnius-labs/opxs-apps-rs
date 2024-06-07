use std::{path::Path, sync::Arc};

use chrono::{Duration, Utc};
use core_base::clock::Clock;
use core_cloud::aws::s3::S3Client;
use opxs_base::AppError;

use crate::{ImageConvertFile, ImageConvertJobStatus, ImageConvertRequestParam, ImageType};

use super::ImageConvertJobRepository;

pub struct ImageConvertJobCreator {
    pub image_convert_job_repository: Arc<ImageConvertJobRepository>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl ImageConvertJobCreator {
    pub async fn create_image_convert_job(&self, job_id: &str, file_name: &str, typ: &ImageType) -> Result<String, AppError> {
        let file_stem = Path::new(file_name)
            .file_stem()
            .ok_or(anyhow::anyhow!("invalid file_name"))?
            .to_str()
            .ok_or(anyhow::anyhow!("invalid file_name"))?;
        let origin_type = ImageType::from_file_name(file_name);

        let input = ImageConvertFile {
            file_name: file_name.to_string(),
            typ: origin_type,
        };
        let output = ImageConvertFile {
            file_name: format!("{}.{}", file_stem, typ.get_extension()),
            typ: typ.clone(),
        };

        let param = ImageConvertRequestParam {
            input: input.clone(),
            output,
        };
        self.image_convert_job_repository.create_image_convert_job(job_id, &param).await?;

        let now = self.clock.now();
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

            let now = self.clock.now();
            let expires_in = Duration::minutes(10);
            let download_uri = self
                .s3_client
                .gen_get_presigned_uri(format!("out/{}", job_id).as_str(), now, expires_in, &param.output.file_name)
                .await?;

            Ok((job.status, Some(download_uri)))
        } else {
            Ok((job.status, None))
        }
    }
}
