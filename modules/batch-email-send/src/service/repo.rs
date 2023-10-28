use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;

use core_base::clock::SystemClock;

use opxs_shared::message::batch::email_send::EmailSendJob;

pub struct EmailSendJobRepository {
    pub db: Arc<PgPool>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
}

impl EmailSendJobRepository {
    pub async fn get_job(&self, id: &i64) -> anyhow::Result<EmailSendJob> {
        let res: EmailSendJob = sqlx::query_as(
            r#"
SELECT *
    FROM email_send_jobs
    WHERE id = $1
"#,
        )
        .bind(id)
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(res)
    }
}
