use std::sync::Arc;

use core_cloud::aws::sqs::SqsSender;

use super::{EmailConfirmRequestParam, EmailSendJobRepository, EmailSendJobSqsMessage};

pub struct EmailSendJobCreator {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
    pub send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
}

impl EmailSendJobCreator {
    #[allow(unused)]
    pub async fn create_email_confirm_job(&self, param: &EmailConfirmRequestParam) -> anyhow::Result<i64> {
        let id = self.email_send_job_repository.create_email_confirm_job(param).await?;
        let message = EmailSendJobSqsMessage { id };
        self.send_email_sqs_sender.send_message(&serde_json::to_string(&message).unwrap()).await?;
        Ok(id)
    }
}
