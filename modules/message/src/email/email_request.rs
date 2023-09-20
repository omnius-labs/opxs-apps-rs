use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct SendMailSimpleText {
    pub to_address: String,
    pub reply_to_addresses: String,
    pub subject: String,
    pub text_body: String,
}
