use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct EmailConfirmRequestParam {
    pub user_name: String,
    pub to_email_address: String,
    pub from_email_address: String,
    pub email_confirm_url: String,
}
