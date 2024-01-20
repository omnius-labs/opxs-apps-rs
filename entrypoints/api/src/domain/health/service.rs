use std::sync::Arc;

use serde_json::{json, Value};

use crate::shared::{error::AppError, info::AppInfo};

use super::repo::WorldRepo;

#[derive(Clone)]
pub struct HealthService {
    pub info: AppInfo,
    pub world_repo: Arc<WorldRepo>,
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Duration;
    use core_base::clock::SystemClockUtc;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    use core_testkit::containers::postgres::PostgresContainer;

    use crate::{
        domain::health::repo::WorldRepo,
        shared::{info::RunMode, world::WorldValidator},
    };

    use super::*;

    #[tokio::test]
    async fn check_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let info = AppInfo {
            mode: RunMode::Local,
            git_semver: "bbb".to_string(),
            git_sha: "ccc".to_string(),
        };

        let system_clock = Arc::new(SystemClockUtc {});
        let world_verifier = WorldValidator { system_clock };
        world_verifier.verify(&info.mode, &container.connection_string).await.unwrap();

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );
        let world_repo = Arc::new(WorldRepo { db });
        let service = HealthService { info, world_repo };

        assert_eq!(
            service.check().await.unwrap(),
            json!({
                "mode": "local",
                "git_semver": "bbb",
                "git_sha": "ccc"
            })
        );
    }
}
