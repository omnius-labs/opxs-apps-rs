use std::sync::Arc;

use chrono::{Duration, Utc};
use serde::Serialize;

use omnius_core_base::clock::Clock;
use omnius_core_cloud::aws::s3::S3Client;

use crate::{FileConvertJobStatus, FileConvertJobType, prelude::*};

use super::FileConvertJobRepository;

pub struct FileConvertJobCreator {
    pub file_convert_job_repository: Arc<FileConvertJobRepository>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl FileConvertJobCreator {
    pub async fn create_job<TParam>(
        &self,
        job_id: &str,
        user_id: &str,
        typ: &FileConvertJobType,
        param: &TParam,
        in_file_name: &str,
        out_file_name: &str,
    ) -> Result<String>
    where
        TParam: ?Sized + Serialize,
    {
        self.file_convert_job_repository
            .create_job(job_id, user_id, typ, param, in_file_name, out_file_name)
            .await?;

        let now = self.clock.now();
        let expires_in = Duration::minutes(5);
        let upload_uri = self
            .s3_client
            .gen_put_presigned_uri(format!("in/{job_id}").as_str(), now, expires_in)
            .await?;

        self.file_convert_job_repository.update_status_to_waiting(job_id).await?;

        Ok(upload_uri)
    }

    pub async fn get_download_url(&self, job_id: &str, user_id: &str) -> Result<(FileConvertJobStatus, Option<String>)> {
        let job = self.file_convert_job_repository.get_job_by_user_id(job_id, user_id).await?;

        if job.status != FileConvertJobStatus::Completed {
            return Ok((job.status, None));
        }

        let now = self.clock.now();
        let expires_in = Duration::minutes(10);
        let download_uri = self
            .s3_client
            .gen_get_presigned_uri(format!("out/{job_id}").as_str(), now, expires_in, &job.out_file_name)
            .await?;

        Ok((job.status, Some(download_uri)))
    }
}
