use chrono::Duration;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::Row;

use super::AppError;

pub struct WorldValidator;

impl WorldValidator {
    pub async fn verify(&self, mode: &str, postgres_url: &str) -> Result<WorldValidatedStatus, AppError> {
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
            let sql = "
CREATE TABLE _world (
    key VARCHAR(255) NOT NULL PRIMARY KEY,
    value TEXT NOT NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
";
            sqlx::query(sql).execute(&db).await?;
        }

        let row: Result<PgRow, sqlx::Error> = sqlx::query("SELECT (value) FROM _world WHERE key = 'mode'").fetch_one(&db).await;

        match row {
            Ok(row) => {
                let got_mode: String = row.get(0);
                if mode != got_mode {
                    return Err(AppError::WorldMismatchError);
                }
                Ok(WorldValidatedStatus::Match)
            }
            Err(sqlx::Error::RowNotFound) => {
                let sql = format!("INSERT INTO _world (key, value) VALUES ('mode', '{mode}')");
                sqlx::query(&sql).execute(&db).await?;
                Ok(WorldValidatedStatus::Set)
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorldValidatedStatus {
    Match,
    Set,
}

#[cfg(test)]
mod tests {
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use super::*;

    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let world_verifier = WorldValidator {};
        let res = world_verifier.verify("aaa", &container.connection_string).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Set);

        let res = world_verifier.verify("aaa", &container.connection_string).await;
        assert_eq!(res.unwrap(), WorldValidatedStatus::Match);

        let res = world_verifier.verify("bbb", &container.connection_string).await;
        assert!(matches!(res, Err(AppError::WorldMismatchError)));
    }
}