use std::sync::Arc;

use chrono::{Duration, Utc};

use core_base::{clock::SystemClock, random_bytes::RandomBytesProvider};

use opxs_base::{AppError, JwtConfig};

use crate::shared::{jwt, kdf::Kdf};

use super::EmailAuthRepo;

#[derive(Clone)]
pub struct EmailAuthService {
    pub auth_repo: Arc<EmailAuthRepo>,
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

    pub async fn login(&self, email: &str, password: &str) -> Result<String, AppError> {
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

    pub async fn confirm(&self, token: &str) -> Result<String, AppError> {
        let now = self.system_clock.now();
        let claims = jwt::verify(&self.jwt_conf.secret.current, token, now)?;

        let email = claims.sub;
        self.auth_repo.update_email_verified(&email, true).await?;

        let user = self.auth_repo.get_user(&email).await?;

        Ok(user.id)
    }

    // pub async fn unregister(&self, refresh_token: &str) -> Result<(), AppError> {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use sqlx::postgres::PgPoolOptions;

    use core_base::{clock::SystemClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
    use core_migration::postgres::PostgresMigrator;
    use core_testkit::containers::postgres::PostgresContainer;

    use opxs_base::JwtSecretConfig;

    use crate::shared::{self, kdf::KdfAlgorithm};

    use super::*;

    #[tokio::test]
    async fn simple_test() {
        let docker = testcontainers::clients::Cli::default();
        let container = PostgresContainer::new(&docker, shared::POSTGRES_VERSION);

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

        let system_clock = Arc::new(SystemClockUtc {});
        let random_bytes_provider = Arc::new(RandomBytesProviderImpl {});
        let tsid_provider = Arc::new(TsidProviderImpl::new(SystemClockUtc, RandomBytesProviderImpl, 16));
        let auth_repo = Arc::new(EmailAuthRepo {
            db,
            system_clock: system_clock.clone(),
            tsid_provider,
        });
        let jwt_conf = JwtConfig {
            secret: JwtSecretConfig {
                current: "a".to_string(),
                previous: "b".to_string(),
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

        // register
        let token = auth_service.register("name", "test@example.com", "password").await.unwrap();
        assert!(matches!(
            auth_service.login("test@example.com", "password").await,
            Err(AppError::UserNotFound)
        ));
        auth_service.confirm(&token).await.unwrap();

        // login
        assert!(auth_service.login("test@example.com", "password").await.is_ok());
    }
}
