#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Duration;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;

    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::{
        domain::health::service::HealthService,
        infra::health::world::WorldRepoImpl,
        shared::{AppInfo, WorldValidator},
    };

    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let world_verifier = WorldValidator {};
        world_verifier.verify("aaa", &container.connection_string).await.unwrap();

        let info = AppInfo {
            mode: "aaa".to_string(),
            git_semver: "bbb".to_string(),
            git_sha: "ccc".to_string(),
            build_timestamp: "ddd".to_string(),
        };
        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );
        let world_repo = Arc::new(WorldRepoImpl { db });
        let service = HealthService { info, world_repo };

        assert_eq!(
            service.check().await.unwrap(),
            json!({
                "mode": "aaa",
                "git_semver": "bbb",
                "git_sha": "ccc"
            })
        );
    }
}
