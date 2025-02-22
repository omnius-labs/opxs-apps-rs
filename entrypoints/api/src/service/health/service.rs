use std::sync::Arc;

use omnius_opxs_base::{AppError, AppInfo};
use serde_json::{Value, json};

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
    use omnius_core_base::clock::ClockUtc;
    use omnius_opxs_base::{RunMode, WorldValidator};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::{service::health::repo::WorldRepo, shared};

    use super::*;

    #[tokio::test]
    async fn check_test() -> TestResult {
        let container = PostgresContainer::new(shared::POSTGRES_VERSION).await?;

        let info = AppInfo {
            app_name: "app".to_string(),
            mode: RunMode::Local,
            git_tag: "git_tag".to_string(),
        };

        let clock = Arc::new(ClockUtc {});
        let world_verifier = WorldValidator::new(&info, &container.connection_string, clock).await?;
        world_verifier.verify().await?;

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std()?))
                .connect(&container.connection_string)
                .await?,
        );
        let world_repo = Arc::new(WorldRepo { db });
        let service = HealthService { info, world_repo };

        assert_eq!(
            service.check().await?,
            json!({
                "mode": "local",
                "git_tag": "git_tag"
            })
        );

        Ok(())
    }
}
