use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct EmailConfirmRequestParam {
    pub email: String,
    pub user_name: String,
    pub email_confirm_url: String,
}
