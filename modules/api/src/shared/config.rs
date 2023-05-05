use std::sync::Arc;

use anyhow::anyhow;
use config::Config as ConfigToml;
use serde::Deserialize;

use omnius_core::cloud::secret::SecretReader;

const APPLICATION_NAME: &str = "opxs-api";

#[derive(Debug, Deserialize)]
struct AppToml {
    pub postgres: PostgresToml,
    pub jwt: JwtToml,
    pub secret: Option<SecretToml>,
}

#[derive(Debug, Deserialize)]
struct PostgresToml {
    pub host: String,
    pub port: i32,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwtToml {
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecretToml {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtConfig {
    pub secret: String,
}

impl AppConfig {
    pub async fn load(path: &str, secret_reader: Arc<dyn SecretReader>) -> anyhow::Result<Self> {
        let toml = ConfigToml::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;
        let mut toml: AppToml = toml.try_deserialize()?;

        if let Some(secret) = &toml.secret {
            let secret_value = secret_reader.read_value(&secret.id).await?;
            let postgres_user = secret_value
                .get("postgres_user")
                .map(|n| n.as_str().unwrap_or_default().to_string());
            let postgres_password = secret_value
                .get("postgres_password")
                .map(|n| n.as_str().unwrap_or_default().to_string());
            let jwt_secret = secret_value
                .get("jwt_secret")
                .map(|n| n.as_str().unwrap_or_default().to_string());

            if toml.postgres.user.is_none() {
                if let Some(postgres_user) = postgres_user {
                    toml.postgres.user = Some(postgres_user);
                }
            }
            if toml.postgres.password.is_none() {
                if let Some(postgres_password) = postgres_password {
                    toml.postgres.password = Some(postgres_password);
                }
            }
            if toml.jwt.secret.is_none() {
                if let Some(jwt_secret) = jwt_secret {
                    toml.jwt.secret = Some(jwt_secret);
                }
            }
        }

        let postgres_url = format!(
            "postgresql://{host}:{port}/{database}?user={user}&password={password}&application_name={application_name}",
            host = urlencoding::encode(&toml.postgres.host),
            port = toml.postgres.port,
            database = urlencoding::encode(&toml.postgres.database),
            user = urlencoding::encode(toml.postgres.user.as_ref().ok_or_else(|| anyhow!("postgres user not found"))?),
            password = urlencoding::encode(toml.postgres.password.as_ref().ok_or_else(|| anyhow!("postgres password not found"))?),
            application_name = urlencoding::encode(APPLICATION_NAME)
        );
        let jwt_secret = toml
            .jwt
            .secret
            .ok_or_else(|| anyhow!("jwt secret not found"))?;

        Ok(Self {
            postgres: PostgresConfig { url: postgres_url },
            jwt: JwtConfig { secret: jwt_secret },
        })
    }
}

#[cfg(test)]
mod tests {
    use omnius_core::cloud::secret::aws::AwsSecretReader;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn secret_reader_test() {
        let secret_reader = Arc::new(AwsSecretReader::new().await);
        let app_config = AppConfig::load("../../conf/dev", secret_reader)
            .await
            .unwrap();
        println!("{:?}", app_config);
    }
}
