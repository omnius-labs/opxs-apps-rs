use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::{clock::SystemClock, random_bytes::RandomBytesProvider};
use sqlx::PgPool;

use crate::{
    domain::{
        auth::service::{EmailAuthService, GoogleAuthService, GoogleOAuth2ProviderImpl, Kdf, KdfAlgorithm, TokenService, UserService},
        health::service::HealthService,
    },
    infra::{
        auth::{email_auth::EmailAuthRepoImpl, provider_auth::ProviderAuthRepoImpl, token::TokenRepoImpl, user::UserRepoImpl},
        health::world::WorldRepoImpl,
    },
};

use super::{AppConfig, AppInfo};

pub struct AppService {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub health: HealthService,
    pub email_auth: EmailAuthService,
    pub google_auth: GoogleAuthService,
    pub token: TokenService,
    pub user: UserService,
}

impl AppService {
    pub fn new(
        info: &AppInfo,
        conf: &AppConfig,
        db: Arc<PgPool>,
        system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
        random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    ) -> Self {
        Self {
            system_clock: system_clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),
            health: HealthService {
                info: info.clone(),
                world_repo: Arc::new(WorldRepoImpl { db: db.clone() }),
            },
            email_auth: EmailAuthService {
                auth_repo: Arc::new(EmailAuthRepoImpl { db: db.clone() }),
                jwt_conf: conf.jwt.clone(),
                kdf: Kdf::new(KdfAlgorithm::Pbkdf2HmacSha256, 1024),
            },
            google_auth: GoogleAuthService {
                auth_repo: Arc::new(ProviderAuthRepoImpl { db: db.clone() }),
                oauth2_provider: Arc::new(GoogleOAuth2ProviderImpl {}),
                auth_conf: conf.auth.clone(),
            },
            token: TokenService {
                system_clock: system_clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.jwt.clone(),
                token_repo: Arc::new(TokenRepoImpl { db: db.clone() }),
            },
            user: UserService {
                user_repo: Arc::new(UserRepoImpl { db }),
            },
        }
    }
}
