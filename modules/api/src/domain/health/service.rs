use std::sync::Arc;

use serde_json::{json, Value};

use crate::shared::{AppError, AppInfo};

use super::repo::WorldRepo;

#[derive(Clone)]
pub struct HealthService {
    pub info: AppInfo,
    pub world_repo: Arc<dyn WorldRepo + Send + Sync>,
}

impl HealthService {
    pub async fn check(&self) -> Result<Value, AppError> {
        let ret = json!({
            "mode": self.world_repo.get_mode().await?,
            "git_semver": self.info.git_semver,
            "git_sha": self.info.git_sha
        });
        Ok(ret)
    }
}
