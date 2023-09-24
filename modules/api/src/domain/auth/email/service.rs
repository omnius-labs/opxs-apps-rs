use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use omnius_core_base::{clock::SystemClock, random_bytes::RandomBytesProvider};

use crate::{
    common::{jwt, Kdf},
    shared::{AppError, JwtConfig},
};

use super::EmailAuthRepo;

#[derive(Clone)]
pub struct EmailAuthService {
    pub auth_repo: Arc<dyn EmailAuthRepo + Send + Sync>,
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub jwt_conf: JwtConfig,
    pub kdf: Kdf,
}

impl EmailAuthService {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<String, AppError> {
        if self.auth_repo.exist_user(email).await? {
            return Err(AppError::DuplicateEmail);
        }

        let salt = self.kdf.gen_salt()?;
        let password_hash = self.kdf.derive(password, &salt)?;

        self.auth_repo
            .create_user(name, email, &hex::encode(password_hash), &hex::encode(salt))
            .await?;

        let now = self.system_clock.now();

        let sub = email.to_string();
        let expires_in = Duration::minutes(30);
        let token = jwt::sign(&self.jwt_conf.secret.current, &sub, expires_in, now)?;

        Ok(token)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<i64, AppError> {
        if !self.auth_repo.exist_user(email).await? {
            return Err(AppError::UserNotFound);
        }

        let user = self.auth_repo.get_user(email).await?;
        let salt = hex::decode(user.salt).map_err(|e| AppError::UnexpectedError(e.into()))?;
        let password_hash = hex::decode(user.password_hash).map_err(|e| AppError::UnexpectedError(e.into()))?;

        if !self.kdf.verify(password, &salt, &password_hash)? {
            return Err(AppError::WrongPassword);
        }

        Ok(user.id)
    }

    pub async fn confirm(&self, token: &str) -> Result<(), AppError> {
        let claims = jwt::verify(&self.jwt_conf.secret.current, token)?;

        let expired_at: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp_opt(claims.exp, 0).unwrap_or(NaiveDateTime::MIN), Utc);
        let now = self.system_clock.now();
        if expired_at < now {
            return Err(AppError::AccessTokenExpired);
        }

        let email = claims.sub;
        self.auth_repo.update_email_verified(&email, true).await?;

        Ok(())
    }

    // pub async fn unregister(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use omnius_core_base::{clock::SystemClockUtc, random_bytes::RandomBytesProviderImpl};
    use omnius_core_migration::Migrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use crate::{common::KdfAlgorithm, domain::auth::email::repo::EmailAuthRepoImpl, shared::JwtSecretConfig};

    use super::*;

    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, "15.1");

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std().unwrap()))
                .connect(&container.connection_string)
                .await
                .unwrap(),
        );

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
        let migrator = Migrator::new(&container.connection_string, migrations_path, "opxs-api", "")
            .await
            .unwrap();
        migrator.migrate().await.unwrap();

        let system_clock = Arc::new(SystemClockUtc {});
        let random_bytes_provider = Arc::new(RandomBytesProviderImpl {});
        let auth_repo = Arc::new(EmailAuthRepoImpl { db });
        let jwt_conf = JwtConfig {
            secret: JwtSecretConfig {
                current: "a".to_string(),
                retired: "b".to_string(),
            },
        };
        let kdf = Kdf {
            algorithm: KdfAlgorithm::Pbkdf2HmacSha256,
            iterations: 10,
        };

        let auth_service = EmailAuthService {
            auth_repo,
            system_clock,
            random_bytes_provider,
            jwt_conf,
            kdf,
        };

        let token = auth_service.register("name", "test@example.com", "password").await.unwrap();
        assert!(matches!(
            auth_service.login("test@example.com", "password").await,
            Err(AppError::UserNotFound)
        ));
        auth_service.confirm(&token).await.unwrap();
        assert!(auth_service.login("test@example.com", "password").await.is_ok());
    }
}
