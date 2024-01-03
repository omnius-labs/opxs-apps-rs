use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailSendJobBatchSqsMessage {
    pub job_id: String,
    pub batch_id: i32,
}
