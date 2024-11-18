use std::{path::PathBuf, sync::Arc};

use omnius_core_cloud::aws::s3::S3Client;

use crate::{ImageConvertJobRepository, ImageConvertRequestParam, ImageConverter};

pub struct ImageConvertExecutor {
    pub image_converter: Arc<dyn ImageConverter + Send + Sync>,
    pub image_convert_job_repository: Arc<ImageConvertJobRepository>,
    pub s3_client: Arc<dyn S3Client + Send + Sync>,
}

impl ImageConvertExecutor {
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
        let param = job.param.ok_or_else(|| anyhow::anyhow!("param is not found"))?;
        let param = serde_json::from_str::<ImageConvertRequestParam>(&param)?;
        self.execute_image_convert(job_id, &param).await?;

        Ok(())
    }

    async fn execute_image_convert(&self, job_id: &str, param: &ImageConvertRequestParam) -> anyhow::Result<()> {
        let in_file = PathBuf::from(format!("/tmp/in_{}.{}", job_id, param.input.typ.get_extension()));
        let out_file = PathBuf::from(format!("/tmp/out_{}.{}", job_id, param.output.typ.get_extension()));

        self.s3_client.get_object(format!("in/{}", job_id).as_str(), &in_file).await?;

        self.image_converter.convert(&in_file, &out_file).await?;

        self.s3_client.put_object(format!("out/{}", job_id).as_str(), &out_file).await?;

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

    use crate::{shared, ImageConvertJobCreator, ImageConverterMock, ImageType};

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
        let job_id = tsid_provider.lock().gen().to_string();
        let upload_url = job_creator.create_image_convert_job(&job_id, "test.png", &ImageType::Jpg).await.unwrap();
        println!("upload_url: {}", upload_url);

        let executor = ImageConvertExecutor {
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

        Ok(())
    }
}
