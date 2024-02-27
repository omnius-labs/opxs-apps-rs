use std::sync::Arc;

use chrono::{Duration, Utc};
use core_base::clock::SystemClock;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::Row;

use super::info::RunMode;

pub struct WorldValidator {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
}

impl WorldValidator {
    pub async fn verify(&self, mode: &RunMode, postgres_url: &str) -> anyhow::Result<WorldValidatedStatus> {
        let db = PgPoolOptions::new()
            .max_connections(10)
            .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
            .connect(postgres_url)
            .await?;

        let sql = "
SELECT EXISTS (
    SELECT FROM
        pg_tables
    WHERE
        schemaname = 'public' AND tablename  = '_world'
);
";
        let (existed,): (bool,) = sqlx::query_as(sql).fetch_one(&db).await?;

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
            .execute(&db)
            .await?;
        }

        let row: Result<PgRow, sqlx::Error> = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'").fetch_one(&db).await;

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
                    .execute(&db)
                    .await?;
                Ok(WorldValidatedStatus::Init)
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldValidatedStatus {
    Init,
    Match,
}

#[cfg(test)]
mod tests {
    use core_base::clock::SystemClockUtc;
    use core_testkit::containers::postgres::PostgresContainer;

    use super::*;

    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let system_clock = Arc::new(SystemClockUtc {});
        let world_verifier = WorldValidator { system_clock };
        let res = world_verifier.verify(&RunMode::Local, &container.connection_string).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Init);

        let res = world_verifier.verify(&RunMode::Local, &container.connection_string).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Match);

        let res = world_verifier.verify(&RunMode::Dev, &container.connection_string).await;
        assert!(res.is_err());
    }
}
