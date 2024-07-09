use std::sync::Arc;

use chrono::{Duration, Utc};
use omnius_core_base::clock::Clock;
use serde_json::json;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};

use crate::{AppInfo, NotifyConfig};

pub struct WorldValidator {
    info: AppInfo,
    clock: Arc<dyn Clock<Utc> + Send + Sync>,
    db: PgPool,
}

impl WorldValidator {
    pub async fn new(info: &AppInfo, postgres_url: &str, clock: Arc<dyn Clock<Utc> + Send + Sync>) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
            .connect(postgres_url)
            .await?;
        Self::init(&db).await?;

        Ok(Self {
            info: info.clone(),
            clock,
            db,
        })
    }

    async fn init(db: &PgPool) -> anyhow::Result<()> {
        let sql = "
SELECT EXISTS (
    SELECT FROM
        pg_tables
    WHERE
        schemaname = 'public' AND tablename  = '_world'
);
";
        let (existed,): (bool,) = sqlx::query_as(sql).fetch_one(db).await?;

        if !existed {
            sqlx::query(
                "
CREATE TABLE IF NOT EXISTS _world (
    key VARCHAR(255) NOT NULL PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
)",
            )
            .execute(db)
            .await?;
        }

        Ok(())
    }

    pub async fn verify(&self) -> anyhow::Result<WorldValidatedStatus> {
        let row: Result<PgRow, sqlx::Error> = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'").fetch_one(&self.db).await;

        match row {
            Ok(row) => {
                let got_mode: String = row.get(0);
                if self.info.mode.to_string() != got_mode {
                    return Err(anyhow::anyhow!("World mismatch"));
                }
                Ok(WorldValidatedStatus::Match)
            }
            Err(sqlx::Error::RowNotFound) => {
                let now = self.clock.now();
                sqlx::query("INSERT INTO _world (key, value, created_at, updated_at) VALUES ('mode', $1, $2, $3)")
                    .bind(self.info.mode.to_string())
                    .bind(now)
                    .bind(now)
                    .execute(&self.db)
                    .await?;
                Ok(WorldValidatedStatus::Init)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn notify(&self, conf: &NotifyConfig) -> anyhow::Result<()> {
        let now = self.clock.now();
        let res = sqlx::query(
            r#"
INSERT INTO _world (key, value, created_at, updated_at) VALUES ($1, $2, $3, $4)
    ON CONFLICT (key)
    DO UPDATE SET value = $2, updated_at = $4
        WHERE _world.value != $2;
"#,
        )
        .bind(format!("git_tag/{}", self.info.app_name))
        .bind(self.info.git_tag.clone())
        .bind(now)
        .bind(now)
        .execute(&self.db)
        .await?;
        if res.rows_affected() == 0 {
            return Ok(());
        }

        let client = reqwest::Client::new();

        let content = format!(
            "name: {name}\nmode: {mode}\ngit tag: {git_tag}",
            name = self.info.app_name,
            mode = self.info.mode,
            git_tag = self.info.git_tag
        );
        client
            .post(&conf.discord.release_webhook_url)
            .json(&json!({
                "content": content,
                "username": "Opxs Release Bot",
                "avatar_url": "https://www.rust-lang.org/logos/rust-logo-512x512.png",
            }))
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldValidatedStatus {
    Init,
    Match,
}

#[cfg(test)]
mod tests {
    use std::env;

    use omnius_core_base::clock::RealClockUtc;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use crate::{AppConfig, RunMode};

    use super::*;

    #[tokio::test]
    async fn verify_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let info = AppInfo {
            app_name: "app".to_string(),
            mode: RunMode::Dev,
            git_tag: "test".to_string(),
        };
        let clock = Arc::new(RealClockUtc {});

        let world_verifier = WorldValidator::new(&info, &container.connection_string, clock.clone()).await.unwrap();
        let res = world_verifier.verify().await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Init);

        let res = world_verifier.verify().await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Match);

        let info = AppInfo {
            app_name: "app".to_string(),
            mode: RunMode::Local,
            git_tag: "test".to_string(),
        };
        let world_verifier = WorldValidator::new(&info, &container.connection_string, clock).await.unwrap();
        let res = world_verifier.verify().await;
        assert!(res.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn notify_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let info = AppInfo {
            app_name: "app".to_string(),
            mode: RunMode::Local,
            git_tag: "test".to_string(),
        };

        env::set_var("AWS_PROFILE", "bews-dev");
        env::set_var("AWS_REGION", "us-east-1");
        let conf = AppConfig::load(&info).await.unwrap();
        println!("{:?}", conf);

        let clock = Arc::new(RealClockUtc {});

        let world_verifier = WorldValidator::new(&info, &container.connection_string, clock).await.unwrap();
        let res = world_verifier.notify(&conf.notify).await;
        println!("{:?}", res);
        assert!(res.is_ok());

        let res = world_verifier.notify(&conf.notify).await;
        assert!(res.is_ok());
    }
}
