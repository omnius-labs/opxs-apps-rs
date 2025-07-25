use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::{clock::Clock, tsid::TsidProvider};
use parking_lot::Mutex;
use serde::Serialize;
use sqlx::PgPool;

use crate::{FileConvertJob, FileConvertJobStatus, FileConvertJobType, prelude::*};

pub struct FileConvertJobRepository {
    pub db: Arc<PgPool>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub tsid_provider: Arc<Mutex<dyn TsidProvider + Send + Sync>>,
}

impl FileConvertJobRepository {
    pub async fn create_job<TParam>(
        &self,
        job_id: &str,
        user_id: Option<&str>,
        typ: &FileConvertJobType,
        param: &TParam,
        in_file_name: &str,
        out_file_name: &str,
    ) -> Result<()>
    where
        TParam: ?Sized + Serialize,
    {
        let now = self.clock.now();

        sqlx::query(
            r#"
INSERT INTO file_convert_jobs (id, user_id, type, status, param, in_file_name, out_file_name, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);
        "#,
        )
        .bind(job_id)
        .bind(user_id)
        .bind(typ)
        .bind(FileConvertJobStatus::Preparing)
        .bind(&serde_json::to_string(param)?)
        .bind(in_file_name)
        .bind(out_file_name)
        .bind(now)
        .bind(now)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_job(&self, id: &str) -> Result<FileConvertJob> {
        let res: FileConvertJob = sqlx::query_as(
            r#"
SELECT *
    FROM file_convert_jobs
    WHERE id = $1
"#,
        )
        .bind(id)
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(res)
    }

    pub async fn update_status_to_waiting(&self, job_id: &str) -> Result<()> {
        self.update_status(job_id, FileConvertJobStatus::Preparing, FileConvertJobStatus::Waiting)
            .await
    }

    pub async fn update_status_to_processing(&self, job_id: &str) -> Result<()> {
        self.update_status(job_id, FileConvertJobStatus::Waiting, FileConvertJobStatus::Processing)
            .await
    }

    pub async fn update_status_to_completed(&self, job_id: &str) -> Result<()> {
        self.update_status(job_id, FileConvertJobStatus::Processing, FileConvertJobStatus::Completed)
            .await
    }

    async fn update_status(&self, job_id: &str, old_status: FileConvertJobStatus, new_status: FileConvertJobStatus) -> Result<()> {
        let now = self.clock.now();

        let res = sqlx::query(
            r#"
UPDATE file_convert_jobs
    SET status = $3, updated_at = $4
    WHERE id = $1 AND status = $2
"#,
        )
        .bind(job_id)
        .bind(old_status)
        .bind(new_status)
        .bind(now)
        .execute(self.db.as_ref())
        .await?;

        if res.rows_affected() < 1 {
            return Err(Error::new(ErrorKind::DatabaseError).message("no rows affected"));
        }

        Ok(())
    }

    pub async fn update_status_to_failed(&self, job_id: &str, failed_reason: &str) -> Result<()> {
        let now = self.clock.now();

        let res = sqlx::query(
            r#"
UPDATE file_convert_jobs
    SET status = 'Failed', failed_reason = $2, updated_at = $3
    WHERE id = $1 AND status = 'Processing'
"#,
        )
        .bind(job_id)
        .bind(failed_reason)
        .bind(now)
        .execute(self.db.as_ref())
        .await?;

        if res.rows_affected() < 1 {
            return Err(Error::new(ErrorKind::DatabaseError).message("no rows affected"));
        }

        Ok(())
    }
}
