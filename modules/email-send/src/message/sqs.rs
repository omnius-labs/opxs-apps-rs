use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailSendJobSqsMessage {
    pub id: i64,
}
