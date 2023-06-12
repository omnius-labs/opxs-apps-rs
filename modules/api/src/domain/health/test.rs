#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{domain::health::service::HealthService, shared::AppInfo};

    #[tokio::test]
    async fn simple_test() {
        let info = AppInfo {
            mode: "a".to_string(),
            git_semver: "b".to_string(),
            git_sha: "c".to_string(),
            build_timestamp: "d".to_string(),
        };
        let service = HealthService { info };
        assert_eq!(
            service.check().await.unwrap(),
            json!({
                "mode": "a",
                "git_semver": "b",
                "git_sha": "c"
            })
        );
    }
}
