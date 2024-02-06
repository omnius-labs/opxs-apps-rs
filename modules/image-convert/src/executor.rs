use std::sync::Arc;

use core_cloud::aws::s3::S3Client;
use tracing::info;

use crate::{ImageConvertJobRepository, ImageConvertJobType, ImageConvertRequestParam, ImageConverter};

pub struct Executor {
    pub image_convert_job_repository: Arc<ImageConvertJobRepository>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl Executor {
    pub async fn execute(&self, job_ids: &[String]) -> anyhow::Result<()> {
        for m in job_ids.iter() {
            self.execute_one(m).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, job_id: &str) -> anyhow::Result<()> {
        let job = self.image_convert_job_repository.get_job(job_id).await?;

        self.image_convert_job_repository.update_status_to_processing(job_id).await?;

        match job.typ {
            ImageConvertJobType::ImageConvert => {
                let param = job.param.ok_or(anyhow::anyhow!("param is not found"))?;
                let param = serde_json::from_str::<ImageConvertRequestParam>(&param)?;
                self.execute_image_convert(job_id, &param).await
            }
            _ => anyhow::bail!("invalid job type"),
        }
    }

    async fn execute_image_convert(&self, job_id: &str, param: &ImageConvertRequestParam) -> anyhow::Result<()> {
        let in_file = format!("/tmp/in_{}.{}", job_id, param.input.format.get_extension());
        let out_file = format!("/tmp/out_{}.{}", job_id, param.output.format.get_extension());

        info!("----- 1 -----");
        self.s3_client.get_object(&param.input.s3_key, in_file.as_str()).await?;

        info!("----- 2 -----");
        ImageConverter::convert(in_file.as_str(), out_file.as_str()).await?;

        info!("----- 3 -----");
        self.s3_client.put_object(&param.output.s3_key, out_file.as_str()).await?;

        info!("----- 4 -----");
        self.image_convert_job_repository.update_status_to_completed(job_id).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use core_base::{
        clock::SystemClockUtc,
        random_bytes::RandomBytesProviderImpl,
        tsid::{TsidProvider, TsidProviderImpl},
    };
    use core_migration::postgres::PostgresMigrator;
    use core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use core_cloud::aws::s3::S3ClientMock;

    use crate::{ImageConvertJobCreator, ImageFormat};

    use super::*;

    // FIXME: ちゃんと書く
    #[tokio::test]
    #[ignore]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );
        let system_clock = Arc::new(SystemClockUtc {});
        let tsid_provider = Arc::new(TsidProviderImpl::new(SystemClockUtc, RandomBytesProviderImpl, 16));
        let s3_client = Arc::new(S3ClientMock::new());
        s3_client
            .gen_put_presigned_uri_outputs
            .borrow_mut()
            .push_back("https://example.com".to_string());

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let image_convert_job_repository = Arc::new(ImageConvertJobRepository {
            db,
            system_clock: system_clock.clone(),
            tsid_provider: tsid_provider.clone(),
        });

        let job_creator = ImageConvertJobCreator {
            image_convert_job_repository: image_convert_job_repository.clone(),
            system_clock: system_clock.clone(),
            s3_client: s3_client.clone(),
        };
        let job_id = tsid_provider.gen().to_string();
        let upload_url = job_creator
            .create_image_convert_job(&job_id, "test.png", &ImageFormat::Png)
            .await
            .unwrap();
        println!("upload_url: {}", upload_url);

        let executor = Executor {
            image_convert_job_repository,
            s3_client,
        };
        executor.execute(&[job_id]).await.unwrap();
    }
}
