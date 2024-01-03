use std::sync::Arc;

use core_cloud::aws::sqs::SqsSender;

use super::{EmailConfirmRequestParam, EmailSendJobBatch, EmailSendJobBatchSqsMessage, EmailSendJobRepository};

pub struct EmailSendJobCreator {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
}

impl EmailSendJobCreator {
    #[allow(unused)]
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<Vec<EmailSendJobBatch>> {
        let job_id = self.email_send_job_repository.create_email_confirm_job(param).await?;
        let batches = self.email_send_job_repository.get_job_batches(job_id.as_str()).await?;

        let messages: Vec<EmailSendJobBatchSqsMessage> = batches
            .iter()
            .map(|n| EmailSendJobBatchSqsMessage {
                job_id: n.job_id.clone(),
                batch_id: n.batch_id,
            })
            .collect();

        for m in messages {
            self.send_email_sqs_sender.send_message(&serde_json::to_string(&m).unwrap()).await?;
        }

        Ok(batches)
    }
}
