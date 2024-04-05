use std::sync::Arc;

use chrono::{Duration, Utc};
use core_base::clock::SystemClock;
use serde_json::json;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};

use crate::NotifyConfig;

use super::info::RunMode;

pub struct WorldValidator {
    system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    db: PgPool,
}

impl WorldValidator {
    pub async fn new(postgres_url: &str, system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>) -> anyhow::Result<Self> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
            .connect(postgres_url)
            .await?;
        Self::init(&db).await?;

        Ok(Self { system_clock, db })
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

    pub async fn verify(&self, mode: &RunMode) -> anyhow::Result<WorldValidatedStatus> {
        let row: Result<PgRow, sqlx::Error> = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'").fetch_one(&self.db).await;

        match row {
            Ok(row) => {
                let got_mode: String = row.get(0);
                if mode.to_string() != got_mode {
                    return Err(anyhow::anyhow!("World mismatch"));
                }
                Ok(WorldValidatedStatus::Match)
            }
            Err(sqlx::Error::RowNotFound) => {
                let now = self.system_clock.now();
                sqlx::query("INSERT INTO _world (key, value, created_at, updated_at) VALUES ('mode', $1, $2, $3)")
                    .bind(mode.to_string())
                    .bind(now)
                    .bind(now)
                    .execute(&self.db)
                    .await?;
                Ok(WorldValidatedStatus::Init)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn notify(&self, git_tag: &str, conf: &NotifyConfig) -> anyhow::Result<()> {
        let now = self.system_clock.now();
        let res = sqlx::query("INSERT INTO _world (key, value, created_at, updated_at) VALUES ('git_tag', $1, $2, $3)")
            .bind(git_tag.to_string())
            .bind(now)
            .bind(now)
            .execute(&self.db)
            .await?;
        if res.rows_affected() == 0 {
            return Ok(());
        }

        let client = reqwest::Client::new();

        client
            .post(&conf.discord.release_webhook_url)
            .json(&json!({
                "content": format!("git tag: {git_tag} をリリースしました"),
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

    use core_base::clock::SystemClockUtc;
    use core_testkit::containers::postgres::PostgresContainer;

    use crate::AppConfig;

    use super::*;

    #[tokio::test]
    async fn verify_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let system_clock = Arc::new(SystemClockUtc {});
        let world_verifier = WorldValidator::new(&container.connection_string, system_clock).await.unwrap();
        let res = world_verifier.verify(&RunMode::Local).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Init);

        let res = world_verifier.verify(&RunMode::Local).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Match);

        let res = world_verifier.verify(&RunMode::Dev).await;
        assert!(res.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn notify_test() {
        env::set_var("AWS_PROFILE", "opxs-dev");
        env::set_var("AWS_REGION", "us-east-1");

        let app_config = AppConfig::load("test", &RunMode::Dev).await.unwrap();
        println!("{:?}", app_config);

        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let system_clock = Arc::new(SystemClockUtc {});
        let world_verifier = WorldValidator::new(&container.connection_string, system_clock).await.unwrap();
        let res = world_verifier.notify("test", &app_config.notify).await;
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
