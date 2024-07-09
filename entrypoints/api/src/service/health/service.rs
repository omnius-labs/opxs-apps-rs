use std::sync::Arc;

use omnius_opxs_base::{AppError, AppInfo};
use serde_json::{json, Value};

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
            "git_tag": self.info.git_tag,
        });
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Duration;
    use omnius_core_base::clock::RealClockUtc;
    use omnius_opxs_base::{RunMode, WorldValidator};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::service::health::repo::WorldRepo;

    use super::*;

    #[tokio::test]
    async fn check_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let info = AppInfo {
            app_name: "app".to_string(),
            mode: RunMode::Local,
            git_tag: "git_tag".to_string(),
        };

        let clock = Arc::new(RealClockUtc {});
        let world_verifier = WorldValidator::new(&info, &container.connection_string, clock).await.unwrap();
        world_verifier.verify().await.unwrap();

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
                "git_tag": "git_tag"
            })
        );
    }
}
