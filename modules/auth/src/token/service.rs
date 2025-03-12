use std::sync::Arc;

use chrono::{Duration, Utc};
use parking_lot::Mutex;

use omnius_core_base::{clock::Clock, random_bytes::RandomBytesProvider};

use omnius_opxs_base::{AppError, JwtConfig};

use crate::{crypto::jwt, model::AuthToken};

use super::TokenRepo;

const ACCESS_TOKEN_EXPIRES_IN: Duration = Duration::minutes(5);
const REFRESH_TOKEN_EXPIRES_IN: Duration = Duration::days(14);

pub struct TokenService {
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<Mutex<dyn RandomBytesProvider + Send + Sync>>,
    pub jwt_conf: JwtConfig,
    pub token_repo: Arc<TokenRepo>,
}

impl TokenService {
    pub async fn create(&self, user_id: &str) -> Result<AuthToken, AppError> {
        let now = self.clock.now();

        let access_token_expires_at = now + ACCESS_TOKEN_EXPIRES_IN;
        let refresh_token_expires_at = now + REFRESH_TOKEN_EXPIRES_IN;

        let sub = user_id.to_string();
        let access_token = jwt::sign(&self.jwt_conf.secret.current, &sub, access_token_expires_at, now)?;
        let refresh_token = hex::encode(self.random_bytes_provider.lock().get_bytes(32));

        self.token_repo.create_token(user_id, &refresh_token, &refresh_token_expires_at).await?;

        Ok(AuthToken {
            access_token,
            access_token_expires_at: access_token_expires_at.naive_utc(),
            refresh_token,
            refresh_token_expires_at: refresh_token_expires_at.naive_utc(),
        })
    }

    pub async fn delete(&self, user_id: &str) -> Result<(), AppError> {
        self.token_repo.delete_token(user_id).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthToken, AppError> {
        let now = self.clock.now();
        let user_id = self.token_repo.get_user_id(refresh_token).await?;

        let access_token_expires_at = now + ACCESS_TOKEN_EXPIRES_IN;
        let refresh_token_expires_at = now + REFRESH_TOKEN_EXPIRES_IN;

        let sub = user_id.to_string();
        let access_token = jwt::sign(&self.jwt_conf.secret.current, &sub, access_token_expires_at, now)?;
        let refresh_token = hex::encode(self.random_bytes_provider.lock().get_bytes(32));

        self.token_repo.create_token(&user_id, &refresh_token, &refresh_token_expires_at).await?;

        Ok(AuthToken {
            access_token,
            access_token_expires_at: access_token_expires_at.naive_utc(),
            refresh_token,
            refresh_token_expires_at: refresh_token_expires_at.naive_utc(),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration};
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_base::{clock::ClockUtc, random_bytes::RandomBytesProviderImpl};
    use omnius_core_migration::postgres::PostgresMigrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use omnius_opxs_base::JwtSecretConfig;

    use crate::{
        model::{UserAuthenticationType, UserRole},
        shared::POSTGRES_VERSION,
    };

    use super::*;

    #[tokio::test]
    async fn simple_test() -> TestResult {
        let container = PostgresContainer::new(POSTGRES_VERSION).await?;

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let clock = Arc::new(ClockUtc {});
        let token_service = TokenService {
            clock: clock.clone(),
            random_bytes_provider: Arc::new(Mutex::new(RandomBytesProviderImpl::new())),
            jwt_conf: JwtConfig {
                secret: JwtSecretConfig {
                    current: "current".to_string(),
                    previous: "previous".to_string(),
                },
            },
            token_repo: Arc::new(TokenRepo {
                db: db.clone(),
                clock: clock.clone(),
            }),
        };

        let now = DateTime::from_timestamp(0, 0).unwrap();
        let user_id = "test_user_id";
        let user_name = "test_user_name";

        // create user
        sqlx::query(
            r#"
INSERT INTO users (id, name, authentication_type, role, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6)
"#,
        )
        .bind(user_id)
        .bind(user_name)
        .bind(UserAuthenticationType::Email)
        .bind(UserRole::User)
        .bind(now)
        .bind(now)
        .execute(db.as_ref())
        .await
        .unwrap();

        let token = token_service.create(user_id).await.unwrap();

        let token = token_service.refresh(&token.refresh_token).await.unwrap();

        token_service.delete(user_id).await.unwrap();

        assert!(token_service.refresh(&token.refresh_token).await.is_err());

        token_service.delete(user_id).await.unwrap();

        Ok(())
    }
}
