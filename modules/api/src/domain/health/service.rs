use std::env;

use serde_json::{json, Value};

use crate::shared::AppError;

#[derive(Clone)]
pub struct HealthService;

impl HealthService {
    pub async fn check(&self) -> Result<Value, AppError> {
        let mode = env::var("RUN_MODE").map_err(|e| AppError::UnexpectedError(e.into()))?;

        let ret = json!({
            "mode": mode,
            "git_semver": env!("VERGEN_GIT_SEMVER"),
            "git_sha": env!("VERGEN_GIT_SHA")
        });
        Ok(ret)
    }
}
