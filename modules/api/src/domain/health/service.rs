use serde_json::{json, Value};

use crate::shared::{AppError, AppInfo};

#[derive(Clone)]
pub struct HealthService {
    pub info: AppInfo,
}

impl HealthService {
    pub async fn check(&self) -> Result<Value, AppError> {
        let ret = json!({
            "mode": self.info.mode,
            "git_semver": self.info.git_semver,
            "git_sha": self.info.git_sha
        });
        Ok(ret)
    }
}
