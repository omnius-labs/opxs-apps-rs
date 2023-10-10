use omnius_core_cloud::aws::secrets::SecretsReader;

use super::RunMode;

const APPLICATION_NAME: &str = "opxs-batch-email-send";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub ses: SesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SesConfig {
    pub configuration_set_name: String,
    pub from_address: String,
}

impl AppConfig {
    pub async fn load(mode: &RunMode, secret_reader: Box<dyn SecretsReader>) -> anyhow::Result<Self> {
        let secret_value = serde_json::from_str::<serde_json::Value>(&secret_reader.read_value("opxs-batch-email-send").await?)?;
        let postgres_user = secret_value.get_str("postgres_user")?;
        let postgres_password = secret_value.get_str("postgres_password")?;

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
                    ses: SesConfig {
                        configuration_set_name: "opxs-email-send".to_string(),
                        from_address: "no-reply@opxs-dev.omnius-labs.com".to_string(),
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
                    ses: SesConfig {
                        configuration_set_name: "opxs-email-send".to_string(),
                        from_address: "no-reply@opxs-dev.omnius-labs.com".to_string(),
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
    use omnius_core_cloud::aws::secrets::SecretsReaderImpl;

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
