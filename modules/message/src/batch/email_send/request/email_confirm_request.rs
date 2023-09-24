use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct EmailConfirmRequest {
    pub to_address: String,
    pub token: String,
}
