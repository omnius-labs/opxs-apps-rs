use std::sync::Arc;

use core_cloud::aws::s3::S3Client;

use crate::{ImageConvertJobRepository, ImageConvertRequestParam, ImageConverter};

pub struct Executor {
    pub image_converter: Arc<dyn ImageConverter + Send + Sync>,
    pub image_convert_job_repository: Arc<ImageConvertJobRepository>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl Executor {
    pub async fn execute(&self, job_ids: &[String]) -> anyhow::Result<()> {
        for job_id in job_ids.iter() {
            self.image_convert_job_repository.update_status_to_processing(job_id).await?;

            let res = self.execute_one(job_id).await;

            if let Err(e) = res {
                self.image_convert_job_repository
                    .update_status_to_failed(job_id, e.to_string().as_str())
                    .await?;
                continue;
            }

            self.image_convert_job_repository.update_status_to_completed(job_id).await?;
        }
        Ok(())
    }

    async fn execute_one(&self, job_id: &str) -> anyhow::Result<()> {
        let job = self.image_convert_job_repository.get_job(job_id).await?;
        let param = job.param.ok_or(anyhow::anyhow!("param is not found"))?;
        let param = serde_json::from_str::<ImageConvertRequestParam>(&param)?;
        self.execute_image_convert(job_id, &param).await?;

        Ok(())
    }

    async fn execute_image_convert(&self, job_id: &str, param: &ImageConvertRequestParam) -> anyhow::Result<()> {
        let in_file = format!("/tmp/in_{}.{}", job_id, param.input.typ.get_extension());
        let out_file = format!("/tmp/out_{}.{}", job_id, param.output.typ.get_extension());

        self.s3_client.get_object(format!("in/{}", job_id).as_str(), in_file.as_str()).await?;

        self.image_converter.convert(in_file.as_str(), out_file.as_str()).await?;

        self.s3_client.put_object(format!("out/{}", job_id).as_str(), out_file.as_str()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use core_base::{
        clock::RealClockUtc,
        random_bytes::RandomBytesProviderImpl,
        tsid::{TsidProvider, TsidProviderImpl},
    };
    use core_migration::postgres::PostgresMigrator;
    use core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use core_cloud::aws::s3::S3ClientMock;

    use crate::{ImageConvertJobCreator, ImageConverterMock, ImageType};

    use super::*;

    #[tokio::test]
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
        let clock = Arc::new(RealClockUtc {});
        let tsid_provider = Arc::new(TsidProviderImpl::new(RealClockUtc, RandomBytesProviderImpl, 16));
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

        let image_converter = Arc::new(ImageConverterMock::new());
        let image_convert_job_repository = Arc::new(ImageConvertJobRepository {
            db,
            clock: clock.clone(),
            tsid_provider: tsid_provider.clone(),
        });

        let job_creator = ImageConvertJobCreator {
            image_convert_job_repository: image_convert_job_repository.clone(),
            clock: clock.clone(),
            s3_client: s3_client.clone(),
        };
        let job_id = tsid_provider.gen().to_string();
        let upload_url = job_creator.create_image_convert_job(&job_id, "test.png", &ImageType::Jpg).await.unwrap();
        println!("upload_url: {}", upload_url);

        let executor = Executor {
            image_converter: image_converter.clone(),
            image_convert_job_repository,
            s3_client: s3_client.clone(),
        };
        executor.execute(&[job_id.clone()]).await.unwrap();

        println!("{:?}", image_converter.convert_inputs.lock().first().unwrap());
        assert_eq!(s3_client.get_object_inputs.lock().first().unwrap().key, format!("in/{}", job_id).as_str());
        assert_eq!(
            s3_client.put_object_inputs.lock().first().unwrap().key,
            format!("out/{}", job_id).as_str()
        );
    }
}
