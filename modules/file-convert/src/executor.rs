use std::sync::Arc;

use anyhow::Context;
use omnius_core_cloud::aws::s3::S3Client;
use tempfile::tempdir;
use tracing::info;

use crate::{FileConvertImageRequestParam, FileConvertJobRepository, FileConvertJobType, ImageConverter};

pub struct FileConvertExecutor {
    pub file_convert_job_repository: Arc<FileConvertJobRepository>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
    pub image_converter: Arc<dyn ImageConverter + Send + Sync>,
    // pub meta_converters: Arc<dyn ImageConverter + Send + Sync>,
}

impl FileConvertExecutor {
    pub async fn execute(&self, job_ids: &[String]) -> anyhow::Result<()> {
        for job_id in job_ids.iter() {
            info!("Start processing job: {}", job_id);

            self.file_convert_job_repository.update_status_to_processing(job_id).await?;

            let res = self.execute_one(job_id).await;

            if let Err(e) = res {
                self.file_convert_job_repository
                    .update_status_to_failed(job_id, e.to_string().as_str())
                    .await?;
                continue;
            }

            self.file_convert_job_repository.update_status_to_completed(job_id).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, job_id: &str) -> anyhow::Result<()> {
        let job = self.file_convert_job_repository.get_job(job_id).await?;

        match &job.typ {
            FileConvertJobType::Image => {
                let param = job.param.ok_or_else(|| anyhow::anyhow!("param is not found"))?;
                let param = serde_json::from_str::<FileConvertImageRequestParam>(&param)?;

                let working_dir = tempdir()?;

                let in_type = param.in_type.clone();
                let in_path = working_dir.path().join(format!("in_{}", job_id)).with_extension(in_type.to_extension());
                let out_type = param.out_type.clone();
                let out_path = working_dir.path().join(format!("out_{}", job_id)).with_extension(out_type.to_extension());

                self.s3_client
                    .get_object(format!("in/{}", job_id).as_str(), &in_path)
                    .await
                    .context("Failed to download input file from S3")?;

                info!("Start converting image: {:?}", param);

                self.image_converter
                    .convert(in_path.as_path(), &in_type, out_path.as_path(), &out_type)
                    .await?;

                info!("Finish converting image: {:?}", param);

                self.s3_client
                    .put_object(format!("out/{}", job_id).as_str(), &out_path)
                    .await
                    .context("Failed to upload output file to S3")?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use parking_lot::Mutex;
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_base::{
        clock::ClockUtc,
        random_bytes::RandomBytesProviderImpl,
        tsid::{TsidProvider, TsidProviderImpl},
    };
    use omnius_core_cloud::aws::s3::S3ClientMock;
    use omnius_core_migration::postgres::PostgresMigrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::{FileConvertImageInputFileType, FileConvertImageOutputFileType, FileConvertJobCreator, ImageConverterMock, shared};

    use super::*;

    #[tokio::test]
    async fn simple_test() -> TestResult {
        let container = PostgresContainer::new(shared::POSTGRES_VERSION).await?;

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );
        let clock = Arc::new(ClockUtc {});
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));
        let s3_client = Arc::new(S3ClientMock::new());
        s3_client
            .gen_put_presigned_uri_outputs
            .lock()
            .push_back("https://put.s3.example.com".to_string());
        s3_client
            .gen_get_presigned_uri_outputs
            .lock()
            .push_back("https://get.s3.example.com".to_string());

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let file_convert_job_repository = Arc::new(FileConvertJobRepository {
            db,
            clock: clock.clone(),
            tsid_provider: tsid_provider.clone(),
        });

        let job_creator = FileConvertJobCreator {
            file_convert_job_repository: file_convert_job_repository.clone(),
            clock: clock.clone(),
            s3_client: s3_client.clone(),
        };
        let job_id = tsid_provider.lock().create().to_string();
        let param = FileConvertImageRequestParam {
            in_type: FileConvertImageInputFileType::Jpg,
            out_type: FileConvertImageOutputFileType::Png,
        };
        let upload_url = job_creator
            .create_job(&job_id, None, &FileConvertJobType::Image, &param, "test.jpg", "test.png")
            .await
            .unwrap();
        println!("upload_url: {}", upload_url);

        let image_converter = Arc::new(ImageConverterMock::new());
        let executor = FileConvertExecutor {
            file_convert_job_repository: file_convert_job_repository.clone(),
            s3_client: s3_client.clone(),
            image_converter: image_converter.clone(),
        };
        executor.execute(&[job_id.clone()]).await.unwrap();

        println!("{:?}", image_converter.convert_inputs.lock().first().unwrap());
        assert_eq!(s3_client.get_object_inputs.lock().first().unwrap().key, format!("in/{}", job_id).as_str());
        assert_eq!(
            s3_client.put_object_inputs.lock().first().unwrap().key,
            format!("out/{}", job_id).as_str()
        );

        Ok(())
    }
}
