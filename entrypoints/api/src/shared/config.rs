use core_cloud::aws::secrets::SecretsReader;
use opxs_auth::shared::config::{AuthConfig, GoogleAuthConfig, JwtConfig, JwtSecretConfig};

use super::info::RunMode;

const APPLICATION_NAME: &str = "opxs-api";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub jwt: JwtConfig,
    pub auth: AuthConfig,
    pub web: WebConfig,
    pub email: EmailConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebConfig {
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConfig {
    pub from_email_address: String,
}

impl AppConfig {
    pub async fn load(mode: &RunMode, secret_reader: Box<dyn SecretsReader>) -> anyhow::Result<Self> {
        let secret_value = serde_json::from_str::<serde_json::Value>(&secret_reader.read_value("opxs-api").await?)?;
        let postgres_user = secret_value.get_str("postgres_user")?;
        let postgres_password = secret_value.get_str("postgres_password")?;
        let jwt_secret_current = secret_value.get_str("jwt_secret_current")?;
        let jwt_secret_retired = secret_value.get_str("jwt_secret_retired")?;
        let auth_google_client_id = secret_value.get_str("auth_google_client_id")?;
        let auth_google_client_secret = secret_value.get_str("auth_google_client_secret")?;

        match mode {
            RunMode::Local => {
                let postgres_url = format!(
                    "postgresql://{host}:{port}/{database}?user={user}&password={password}&application_name={application_name}",
                    host = urlencoding::encode("localhost"),
                    port = 15432,
                    database = "local_opxs",
                    user = "postgres",
                    password = "postgres",
                    application_name = urlencoding::encode(APPLICATION_NAME)
                );

                Ok(Self {
                    postgres: PostgresConfig { url: postgres_url },
                    jwt: JwtConfig {
                        secret: JwtSecretConfig {
                            current: "current".to_string(),
                            retired: "refired".to_string(),
                        },
                    },
                    auth: AuthConfig {
                        google: GoogleAuthConfig {
                            client_id: auth_google_client_id,
                            client_secret: auth_google_client_secret,
                        },
                    },
                    web: WebConfig {
                        origin: "https://localhost.omnius-labs.com/".to_string(),
                    },
                    email: EmailConfig {
                        from_email_address: "Opxs <no-reply@opxs-dev.omnius-labs.com>".to_string(),
                    },
                })
            }
            RunMode::Dev => {
                let postgres_url = format!(
                    "postgresql://{host}:{port}/{database}?user={user}&password={password}&application_name={application_name}",
                    host = urlencoding::encode("tk2-223-21081.vs.sakura.ne.jp"),
                    port = 15432,
                    database = "dev_opxs",
                    user = postgres_user,
                    password = postgres_password,
                    application_name = urlencoding::encode(APPLICATION_NAME)
                );

                Ok(Self {
                    postgres: PostgresConfig { url: postgres_url },
                    jwt: JwtConfig {
                        secret: JwtSecretConfig {
                            current: jwt_secret_current,
                            retired: jwt_secret_retired,
                        },
                    },
                    auth: AuthConfig {
                        google: GoogleAuthConfig {
                            client_id: auth_google_client_id,
                            client_secret: auth_google_client_secret,
                        },
                    },
                    web: WebConfig {
                        origin: "https://opxs-dev.omnius-labs.com/".to_string(),
                    },
                    email: EmailConfig {
                        from_email_address: "Opxs <no-reply@opxs-dev.omnius-labs.com>".to_string(),
                    },
                })
            }
        }
    }
}

trait ValueExt {
    fn get_str(&self, name: &str) -> anyhow::Result<String>;
}

impl ValueExt for serde_json::Value {
    fn get_str(&self, name: &str) -> anyhow::Result<String> {
        let res = self
            .get(name)
            .map(|n| n.as_str().unwrap_or_default().to_string())
            .ok_or(anyhow::anyhow!("{name} is not found"))?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use core_cloud::aws::secrets::SecretsReaderImpl;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn secret_reader_test() {
        let sdk_config = aws_config::load_from_env().await;
        let secret_reader = Box::new(SecretsReaderImpl {
            client: aws_sdk_secretsmanager::Client::new(&sdk_config),
        });
        let app_config = AppConfig::load(&RunMode::Dev, secret_reader).await.unwrap();
        println!("{:?}", app_config);
    }
}
