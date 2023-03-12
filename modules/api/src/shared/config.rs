use std::sync::Arc;

use anyhow::anyhow;
use config::Config as ConfigToml;
use serde::Deserialize;

use crate::domain::secret::service::SecretReader;

const APPLICATION_NAME: &str = "opxs-api";

#[derive(Debug, Deserialize)]
struct AppToml {
    pub postgres: PostgresToml,
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
struct SecretToml {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub url: String,
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
            let postgres_user = secret_value.get("postgres/user").map(|n| n.to_string());
            let postgres_password = secret_value.get("postgres/password").map(|n| n.to_string());

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
        }

        let postgres_url = format!(
            "postgresql://{host}:{port}/{database}?user={user}&password={password}&application_name={application_name}",
            host = toml.postgres.host,
            port = toml.postgres.port,
            database = toml.postgres.database,
            user = toml.postgres.user.as_ref().ok_or_else(|| anyhow!("'postgres user not found'"))?,
            password = toml.postgres.password.as_ref().ok_or_else(|| anyhow!("'postgres password not found'"))?,
            application_name = APPLICATION_NAME
        );

        Ok(Self {
            postgres: PostgresConfig { url: postgres_url },
        })
    }
}
