use std::sync::Arc;

use crate::{
    provider::ProviderAuthRepo,
    shared::{config::AuthConfig, error::AuthError},
};

use super::GoogleOAuth2Provider;

#[derive(Clone)]
pub struct GoogleAuthService {
    pub oauth2_provider: Arc<dyn GoogleOAuth2Provider + Send + Sync>,
    pub auth_repo: Arc<ProviderAuthRepo>,
    pub auth_conf: AuthConfig,
}

impl GoogleAuthService {
    pub async fn register(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<String, AuthError> {
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
            return Err(AuthError::RegisterRejection(anyhow::anyhow!("Nonce mismatch error")));
        }

        if let Ok(user) = self.auth_repo.get_user("google", &id_token_claims.sub).await {
            return Ok(user.id);
        }

        let user_info = self.oauth2_provider.get_user_info(&access_token).await?;

        let user_id = self.auth_repo.create_user(&user_info.name, "google", &id_token_claims.sub).await?;

        Ok(user_id)
    }

    pub async fn login(&self, auth_code: &str, auth_redirect_uri: &str, auth_nonce: &str) -> Result<String, AuthError> {
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
            return Err(AuthError::LoginRejection(anyhow::anyhow!("Nonce mismatch error")));
        }

        let user = self.auth_repo.get_user("google", &id_token_claims.sub).await?;

        Ok(user.id)
    }

    // pub async fn unregister(&self, refresh_token: &str) -> Result<(), AuthError> {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use async_trait::async_trait;
    use chrono::Duration;
    use core_base::{clock::SystemClockUtc, random_bytes::RandomBytesProviderImpl, tsid::TsidProviderImpl};
    use core_migration::postgres::PostgresMigrator;
    use core_testkit::containers::postgres::PostgresContainer;
    use sqlx::postgres::PgPoolOptions;

    use crate::{
        provider::{IdTokenClaims, OAuth2TokenResult, UserInfo},
        shared::{self, config::GoogleAuthConfig},
    };

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

        let access_token = "access_token";
        let provider_user_id = "provider_user_id";
        let nonce = "nonce";

        let user_name = "user_name";
        let user_email = "user_email";

        let code = "auth_code";
        let redirect_uri = "auth_redirect_uri";

        let client_id = "client_id";
        let client_secret = "client_secret";

        let system_clock = Arc::new(SystemClockUtc {});
        let tsid_provider = Arc::new(TsidProviderImpl::new(SystemClockUtc, RandomBytesProviderImpl, 16));
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
            system_clock: system_clock.clone(),
            tsid_provider,
        });
        let auth_conf = AuthConfig {
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
        let user_id = auth_service.register(code, redirect_uri, nonce).await.unwrap();
        println!("{}", user_id);
        assert_eq!(*oauth2_provider.clone().get_oauth2_token_param.borrow().code, code.to_string());
        assert_eq!(
            *oauth2_provider.clone().get_oauth2_token_param.borrow().redirect_uri,
            redirect_uri.to_string()
        );
        assert_eq!(*oauth2_provider.clone().get_oauth2_token_param.borrow().client_id, client_id.to_string());
        assert_eq!(
            *oauth2_provider.clone().get_oauth2_token_param.borrow().client_secret,
            client_secret.to_string()
        );

        // login
        assert_eq!(auth_service.login(code, redirect_uri, nonce).await.unwrap(), user_id);

        // get user
        let user = auth_repo.get_user("google", provider_user_id).await.unwrap();
        assert_eq!(user.name, user_name.to_string());
    }

    #[derive(Debug, Clone)]
    struct GoogleOAuth2ProviderMock {
        get_oauth2_token_param: RefCell<GetOauth2TokenParam>,
        get_oauth2_token_result: OAuth2TokenResult,
        get_user_info_param: RefCell<String>,
        get_user_info_result: UserInfo,
    }

    unsafe impl Sync for GoogleOAuth2ProviderMock {}
    unsafe impl Send for GoogleOAuth2ProviderMock {}

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
                get_oauth2_token_param: RefCell::new(GetOauth2TokenParam {
                    code: "".to_string(),
                    redirect_uri: "".to_string(),
                    client_id: "".to_string(),
                    client_secret: "".to_string(),
                }),
                get_oauth2_token_result,
                get_user_info_param: RefCell::new("".to_string()),
                get_user_info_result,
            }
        }
    }

    #[async_trait]
    impl GoogleOAuth2Provider for GoogleOAuth2ProviderMock {
        async fn get_oauth2_token(
            &self,
            code: &str,
            redirect_uri: &str,
            client_id: &str,
            client_secret: &str,
        ) -> Result<OAuth2TokenResult, AuthError> {
            *self.get_oauth2_token_param.borrow_mut() = GetOauth2TokenParam {
                code: code.to_string(),
                redirect_uri: redirect_uri.to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            };
            return Ok(self.get_oauth2_token_result.clone());
        }

        async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, AuthError> {
            *self.get_user_info_param.borrow_mut() = access_token.to_string();
            return Ok(self.get_user_info_result.clone());
        }
    }
}