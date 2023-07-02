use anyhow::anyhow;
use config::Config as ConfigToml;
use serde::Deserialize;

use omnius_core_cloud::secret::SecretReader;

const APPLICATION_NAME: &str = "opxs-api";

#[derive(Debug, Deserialize)]
struct AppToml {
    pub postgres: PostgresToml,
    pub jwt: Option<JwtToml>,
    pub auth: Option<AuthToml>,
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
    pub secret: String,
}

#[derive(Debug, Deserialize)]
struct AuthToml {
    pub google: GoogleAuthToml,
}

#[derive(Debug, Deserialize)]
struct GoogleAuthToml {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
struct SecretToml {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub jwt: JwtConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    pub google: GoogleAuthConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl AppConfig {
    pub async fn load(path: &str, secret_reader: Box<dyn SecretReader>) -> anyhow::Result<Self> {
        let toml = ConfigToml::builder().add_source(config::File::with_name(path)).build()?;
        let mut toml: AppToml = toml.try_deserialize()?;

        if let Some(secret) = &toml.secret {
            let secret_value = secret_reader.read_value(&secret.id).await?;
            let postgres_user = secret_value.get("postgres_user").map(|n| n.as_str().unwrap_or_default().to_string());
            let postgres_password = secret_value.get("postgres_password").map(|n| n.as_str().unwrap_or_default().to_string());
            let jwt_secret = secret_value.get("jwt_secret").map(|n| n.as_str().unwrap_or_default().to_string());
            let auth_google_client_id = secret_value
                .get("auth_google_client_id")
                .map(|n| n.as_str().unwrap_or_default().to_string());
            let auth_google_client_secret = secret_value
                .get("auth_google_client_secret")
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
            if toml.jwt.is_none() {
                if let Some(jwt_secret) = jwt_secret {
                    toml.jwt = Some(JwtToml { secret: jwt_secret });
                }
            }
            if toml.auth.is_none() && auth_google_client_id.is_some() && auth_google_client_secret.is_some() {
                toml.auth = Some(AuthToml {
                    google: GoogleAuthToml {
                        client_id: auth_google_client_id.unwrap(),
                        client_secret: auth_google_client_secret.unwrap(),
                    },
                });
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
        let jwt_secret = toml.jwt.map(|n| n.secret).ok_or_else(|| anyhow!("jwt secret not found"))?;
        let auth_google_client_id = toml
            .auth
            .as_ref()
            .map(|n| n.google.client_id.clone())
            .ok_or_else(|| anyhow!("google auth client id not found"))?;
        let auth_google_client_secret = toml
            .auth
            .as_ref()
            .map(|n| n.google.client_secret.clone())
            .ok_or_else(|| anyhow!("google auth client secret not found"))?;

        Ok(Self {
            postgres: PostgresConfig { url: postgres_url },
            jwt: JwtConfig { secret: jwt_secret },
            auth: AuthConfig {
                google: GoogleAuthConfig {
                    client_id: auth_google_client_id,
                    client_secret: auth_google_client_secret,
                },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use omnius_core_cloud::secret::aws::AwsSecretReader;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn secret_reader_test() {
        let secret_reader = Box::new(AwsSecretReader::new().await.unwrap());
        let app_config = AppConfig::load("../../conf/dev", secret_reader).await.unwrap();
        println!("{:?}", app_config);
    }
}
