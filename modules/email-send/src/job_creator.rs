use std::sync::Arc;

use core_cloud::aws::sqs::SqsSender;

use super::{EmailConfirmRequestParam, EmailSendJobBatchSqsMessage, EmailSendJobRepository};

pub struct EmailSendJobCreator {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
}

impl EmailSendJobCreator {
    pub async fn create_email_confirm_job(
        &self,
        job_id: &str,
        user_name: &str,
        to_email_address: &str,
        from_email_address: &str,
        email_confirm_url: &str,
    ) -> anyhow::Result<()> {
        let param = EmailConfirmRequestParam {
            user_name: user_name.to_string(),
            to_email_address: to_email_address.to_string(),
            from_email_address: from_email_address.to_string(),
            email_confirm_url: email_confirm_url.to_string(),
        };
        self.email_send_job_repository.create_email_confirm_job(job_id, &param).await?;
        let batches = self.email_send_job_repository.get_job_batches(job_id).await?;

        let messages: Vec<EmailSendJobBatchSqsMessage> = batches
            .iter()
            .map(|n| EmailSendJobBatchSqsMessage {
                job_id: n.job_id.clone(),
                batch_id: n.batch_id,
            })
            .collect();

        self.email_send_job_repository.update_status_to_waiting(job_id).await?;

        for m in messages.iter() {
            self.send_email_sqs_sender.send_message(&serde_json::to_string(m).unwrap()).await?;
        }

        Ok(())
    }
}
