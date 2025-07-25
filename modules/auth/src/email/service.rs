use std::sync::Arc;

use chrono::{Duration, Utc};
use parking_lot::Mutex;

use omnius_core_base::{clock::Clock, random_bytes::RandomBytesProvider};

use omnius_opxs_base::JwtConfig;

use crate::{
    crypto::{jwt, kdf::Kdf},
    prelude::*,
};

use super::EmailAuthRepo;

#[derive(Clone)]
pub struct EmailAuthService {
    pub auth_repo: Arc<EmailAuthRepo>,
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<Mutex<dyn RandomBytesProvider + Send + Sync>>,
    pub jwt_conf: JwtConfig,
    pub kdf: Kdf,
}

impl EmailAuthService {
    pub async fn register(&self, name: &str, email: &str, password: &str) -> Result<String> {
        if self.auth_repo.exist_user(email).await? {
            return Err(Error::new(ErrorKind::Duplicated).message("duplicated email"));
        }

        let salt = self.kdf.gen_salt()?;
        let password_hash = self.kdf.derive(password, &salt)?;

        self.auth_repo
            .create_user(name, email, &hex::encode(password_hash), &hex::encode(salt))
            .await?;

        let now = self.clock.now();
        let sub = email.to_string();
        let exp = now + Duration::minutes(30);
        let token = jwt::sign(&self.jwt_conf.secret.current, &sub, exp, now)?;

        Ok(token)
    }

    pub async fn unregister(&self, id: &str) -> Result<()> {
        self.auth_repo.delete_user(id).await?;
        Ok(())
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String> {
        if !self.auth_repo.exist_user(email).await? {
            return Err(Error::new(ErrorKind::NotFound).message("user not found"));
        }

        let user = self.auth_repo.get_user(email).await?;
        let salt = hex::decode(user.salt)?;
        let password_hash = hex::decode(user.password_hash)?;

        if !self.kdf.verify(password, &salt, &password_hash)? {
            return Err(Error::new(ErrorKind::Unauthorized).message("invalid password"));
        }

        Ok(user.id)
    }

    pub async fn confirm(&self, token: &str) -> Result<String> {
        let now = self.clock.now();
        let claims = jwt::verify(&self.jwt_conf.secret.current, token, now)?;

        let email = claims.sub;
        self.auth_repo.update_email_verified(&email, true).await?;

        let user = self.auth_repo.get_user(&email).await?;

        Ok(user.id)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use parking_lot::Mutex;
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_base::{clock::ClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
    use omnius_core_migration::postgres::PostgresMigrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use omnius_opxs_base::{JwtSecretConfig, shared::POSTGRES_VERSION};

    use crate::crypto::kdf::KdfAlgorithm;

    use super::*;

    #[tokio::test]
    async fn simple_test() -> TestResult {
        let container = PostgresContainer::new(POSTGRES_VERSION).await?;

        let db = Arc::new(
            PgPoolOptions::new()
                .max_connections(100)
                .idle_timeout(Some(Duration::minutes(15).to_std()?))
                .connect(&container.connection_string)
                .await?,
        );

        let migrations_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../conf/migrations");
        let migrator = PostgresMigrator::new(&container.connection_string, migrations_path, "opxs-api", "").await?;
        migrator.migrate().await?;

        let user_name = "user_name";
        let user_email = "user_email";
        let password = "password";
        let invalid_password = "invalid_password";

        let clock = Arc::new(ClockUtc {});
        let random_bytes_provider = Arc::new(Mutex::new(RandomBytesProviderImpl::new()));
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));
        let auth_repo = Arc::new(EmailAuthRepo {
            db,
            clock: clock.clone(),
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
            auth_repo: auth_repo.clone(),
            clock,
            random_bytes_provider,
            jwt_conf,
            kdf,
        };

        // register
        let token = auth_service.register(user_name, user_email, password).await?;
        assert_eq!(*auth_service.login(user_email, password).await.unwrap_err().kind(), ErrorKind::NotFound);
        auth_service.confirm(&token).await?;

        assert_eq!(
            *auth_service.login(user_email, invalid_password).await.unwrap_err().kind(),
            ErrorKind::Unauthorized
        );

        // login
        assert!(auth_service.login(user_email, password).await.is_ok());

        // get user
        let user = auth_repo.get_user(user_email).await?;
        assert_eq!(user.name, user_name.to_string());

        // unregister
        assert!(auth_service.unregister(user.id.as_str()).await.is_ok());

        // login
        assert_eq!(*auth_service.login(user_email, password).await.unwrap_err().kind(), ErrorKind::NotFound);

        Ok(())
    }
}
