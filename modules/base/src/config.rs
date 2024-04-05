use core_cloud::aws::secrets::{SecretsReader, SecretsReaderImpl};

use super::info::RunMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub web: WebConfig,
    pub auth: AuthConfig,
    pub email: EmailConfig,
    pub image_convert: ImageConvertConfig,
    pub notify: NotifyConfig,
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
pub struct JwtSecretConfig {
    pub current: String,
    pub previous: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub google: GoogleAuthConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtConfig {
    pub secret: JwtSecretConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConfig {
    pub from_email_address: String,
    pub ses: SesConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SesConfig {
    pub configuration_set_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageConvertConfig {
    pub s3: S3Config,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3Config {
    pub bucket: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyConfig {
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordConfig {
    pub release_webhook_url: String,
}

impl AppConfig {
    pub async fn load(application_name: &str, mode: &RunMode) -> anyhow::Result<Self> {
        let secret_reader = Box::new(SecretsReaderImpl {
            client: aws_sdk_secretsmanager::Client::new(&aws_config::load_from_env().await),
        });

        let secret_value = serde_json::from_str::<serde_json::Value>(&secret_reader.read_value("opxs").await?)?;
        let postgres_user = secret_value.get_str("postgres_user")?;
        let postgres_password = secret_value.get_str("postgres_password")?;
        let jwt_secret_current = secret_value.get_str("jwt_secret_current")?;
        let jwt_secret_retired = secret_value.get_str("jwt_secret_retired")?;
        let auth_google_client_id = secret_value.get_str("auth_google_client_id")?;
        let auth_google_client_secret = secret_value.get_str("auth_google_client_secret")?;
        let discord_release_webhook_url = secret_value.get_str("discord_release_webhook_url")?;

        match mode {
            RunMode::Local => {
                let postgres_url = format!(
                    "postgresql://{host}:{port}/{database}?user={user}&password={password}&application_name={application_name}",
                    host = urlencoding::encode("localhost"),
                    port = 15432,
                    database = "local_opxs",
                    user = "postgres",
                    password = "postgres",
                );

                Ok(Self {
                    postgres: PostgresConfig { url: postgres_url },
                    web: WebConfig {
                        origin: "https://localhost.omnius-labs.com/".to_string(),
                    },
                    auth: AuthConfig {
                        jwt: JwtConfig {
                            secret: JwtSecretConfig {
                                current: "current".to_string(),
                                previous: "refired".to_string(),
                            },
                        },
                        google: GoogleAuthConfig {
                            client_id: auth_google_client_id,
                            client_secret: auth_google_client_secret,
                        },
                    },
                    email: EmailConfig {
                        from_email_address: "Opxs <no-reply@opxs-dev.omnius-labs.com>".to_string(),
                        ses: SesConfig {
                            configuration_set_name: "opxs-dev".to_string(),
                        },
                    },
                    image_convert: ImageConvertConfig {
                        s3: S3Config {
                            bucket: "opxs.v1.dev.image-convert".to_string(),
                        },
                    },
                    notify: NotifyConfig {
                        discord: DiscordConfig {
                            release_webhook_url: discord_release_webhook_url,
                        },
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
                );

                Ok(Self {
                    postgres: PostgresConfig { url: postgres_url },
                    web: WebConfig {
                        origin: "https://opxs-dev.omnius-labs.com/".to_string(),
                    },
                    auth: AuthConfig {
                        jwt: JwtConfig {
                            secret: JwtSecretConfig {
                                current: jwt_secret_current,
                                previous: jwt_secret_retired,
                            },
                        },
                        google: GoogleAuthConfig {
                            client_id: auth_google_client_id,
                            client_secret: auth_google_client_secret,
                        },
                    },
                    email: EmailConfig {
                        from_email_address: "Opxs <no-reply@opxs-dev.omnius-labs.com>".to_string(),
                        ses: SesConfig {
                            configuration_set_name: "opxs-dev".to_string(),
                        },
                    },
                    image_convert: ImageConvertConfig {
                        s3: S3Config {
                            bucket: "opxs.v1.dev.image-convert".to_string(),
                        },
                    },
                    notify: NotifyConfig {
                        discord: DiscordConfig {
                            release_webhook_url: discord_release_webhook_url,
                        },
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
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn secret_reader_test() {
        let app_config = AppConfig::load("test", &RunMode::Dev).await.unwrap();
        println!("{:?}", app_config);
    }
}
