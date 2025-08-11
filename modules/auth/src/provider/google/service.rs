use std::sync::Arc;

use omnius_opxs_base::AuthConfig;

use crate::{prelude::*, provider::ProviderAuthRepo};

use super::GoogleOAuth2Provider;

#[derive(Clone)]
pub struct GoogleAuthService {
    pub oauth2_provider: Arc<dyn GoogleOAuth2Provider + Send + Sync>,
    pub auth_repo: Arc<ProviderAuthRepo>,
    pub auth_conf: AuthConfig,
}

impl GoogleAuthService {
    pub async fn register(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<String> {
        let oauth2_token_result = self
            .oauth2_provider
            .get_oauth2_token(
                auth_code,
                auth_redirect_uri,
                &self.auth_conf.google.client_id,
                &self.auth_conf.google.client_secret,
            )
            .await?;
        let access_token = oauth2_token_result.access_token;
        let id_token_claims = oauth2_token_result.id_token_claims;

        if auth_nonce != id_token_claims.nonce {
            return Err(Error::builder().kind(ErrorKind::Unauthorized).message("Nonce mismatch error").build());
        }

        if let Ok(user) = self.auth_repo.get_user("google", &id_token_claims.sub).await {
            return Ok(user.id);
        }

        let user_info = self.oauth2_provider.get_user_info(&access_token).await?;

        let user_id = self.auth_repo.create_user(&user_info.name, "google", &id_token_claims.sub).await?;

        Ok(user_id)
    }

    pub async fn unregister(&self, id: &str) -> Result<()> {
        self.auth_repo.delete_user(id).await?;
        Ok(())
    }

    pub async fn login(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<String> {
        let oauth2_token_result = self
            .oauth2_provider
            .get_oauth2_token(
                auth_code,
                auth_redirect_uri,
                &self.auth_conf.google.client_id,
                &self.auth_conf.google.client_secret,
            )
            .await?;
        let id_token_claims = oauth2_token_result.id_token_claims;

        if auth_nonce != id_token_claims.nonce {
            return Err(Error::builder().kind(ErrorKind::Unauthorized).message("Nonce mismatch error").build());
        }

        let user = self.auth_repo.get_user("google", &id_token_claims.sub).await?;

        Ok(user.id)
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Duration;
    use parking_lot::Mutex;
    use sqlx::postgres::PgPoolOptions;
    use testresult::TestResult;

    use omnius_core_base::{clock::ClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
    use omnius_core_migration::postgres::PostgresMigrator;
    use omnius_core_testkit::containers::postgres::PostgresContainer;

    use omnius_opxs_base::{GoogleAuthConfig, JwtConfig, JwtSecretConfig, shared::POSTGRES_VERSION};

    use crate::provider::{IdTokenClaims, OAuth2TokenResult, UserInfo};

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

        let access_token = "access_token";
        let provider_user_id = "provider_user_id";
        let nonce = "nonce";

        let user_name = "user_name";
        let user_email = "user_email";

        let code = "auth_code";
        let redirect_uri = "auth_redirect_uri";

        let client_id = "client_id";
        let client_secret = "client_secret";

        let clock = Arc::new(ClockUtc {});
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));
        let oauth2_provider = Arc::new(GoogleOAuth2ProviderMock::new(
            OAuth2TokenResult {
                access_token: access_token.to_string(),
                id_token_claims: IdTokenClaims {
                    sub: provider_user_id.to_string(),
                    nonce: nonce.to_string(),
                },
            },
            UserInfo {
                name: user_name.to_string(),
                email: user_email.to_string(),
            },
        ));
        let auth_repo = Arc::new(ProviderAuthRepo {
            db,
            clock: clock.clone(),
            tsid_provider,
        });
        let auth_conf = AuthConfig {
            jwt: JwtConfig {
                secret: JwtSecretConfig {
                    current: "current".to_string(),
                    previous: "previous".to_string(),
                },
            },
            google: GoogleAuthConfig {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            },
        };

        let auth_service = GoogleAuthService {
            oauth2_provider: oauth2_provider.clone(),
            auth_repo: auth_repo.clone(),
            auth_conf,
        };

        // register
        let user_id = auth_service.register(code, redirect_uri, nonce).await?;
        println!("{}", user_id);
        assert_eq!(*oauth2_provider.clone().get_oauth2_token_param.lock().code, code.to_string());
        assert_eq!(
            *oauth2_provider.clone().get_oauth2_token_param.lock().redirect_uri,
            redirect_uri.to_string()
        );
        assert_eq!(*oauth2_provider.clone().get_oauth2_token_param.lock().client_id, client_id.to_string());
        assert_eq!(
            *oauth2_provider.clone().get_oauth2_token_param.lock().client_secret,
            client_secret.to_string()
        );

        // login
        assert_eq!(auth_service.login(code, redirect_uri, nonce).await?, user_id);

        // get user
        let user = auth_repo.get_user("google", provider_user_id).await?;
        assert_eq!(user.name, user_name.to_string());

        // unregister
        assert!(auth_service.unregister(&user_id).await.is_ok());

        // get user
        assert!(auth_repo.get_user("google", provider_user_id).await.is_err());

        Ok(())
    }

    #[derive(Debug, Clone)]
    struct GoogleOAuth2ProviderMock {
        get_oauth2_token_param: Arc<Mutex<GetOauth2TokenParam>>,
        get_oauth2_token_result: OAuth2TokenResult,
        get_user_info_param: Arc<Mutex<String>>,
        get_user_info_result: UserInfo,
    }

    #[derive(Debug, Clone)]
    struct GetOauth2TokenParam {
        pub code: String,
        pub redirect_uri: String,
        pub client_id: String,
        pub client_secret: String,
    }

    impl GoogleOAuth2ProviderMock {
        fn new(get_oauth2_token_result: OAuth2TokenResult, get_user_info_result: UserInfo) -> Self {
            Self {
                get_oauth2_token_param: Arc::new(Mutex::new(GetOauth2TokenParam {
                    code: "".to_string(),
                    redirect_uri: "".to_string(),
                    client_id: "".to_string(),
                    client_secret: "".to_string(),
                })),
                get_oauth2_token_result,
                get_user_info_param: Arc::new(Mutex::new("".to_string())),
                get_user_info_result,
            }
        }
    }

    #[async_trait]
    impl GoogleOAuth2Provider for GoogleOAuth2ProviderMock {
        async fn get_oauth2_token(&self, code: &str, redirect_uri: &str, client_id: &str, client_secret: &str) -> Result<OAuth2TokenResult> {
            *self.get_oauth2_token_param.lock() = GetOauth2TokenParam {
                code: code.to_string(),
                redirect_uri: redirect_uri.to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            };
            return Ok(self.get_oauth2_token_result.clone());
        }

        async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
            *self.get_user_info_param.lock() = access_token.to_string();
            return Ok(self.get_user_info_result.clone());
        }
    }
}
