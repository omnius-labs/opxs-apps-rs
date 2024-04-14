use std::sync::Arc;

use crate::{EmailSendJobRepository, SesNotification};

#[allow(dead_code)]
pub struct EmailSendFeedback {
    pub email_send_job_repository: Arc<EmailSendJobRepository>,
}

#[allow(dead_code)]
impl EmailSendFeedback {
    pub async fn feedback(&self, _ses: SesNotification) -> anyhow::Result<()> {
        todo!()
    }
}
