use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::{clock::SystemClock, random_string::RandomStringGenerator};
use sqlx::PgPool;

use crate::{
    domain::{
        auth::service::{EmailAuthService, GoogleAuthService, GoogleOAuth2ProviderImpl, Kdf, KdfAlgorithm, TokenService},
        health::service::HealthService,
    },
    infra::{
        auth::{email_auth::EmailAuthRepoImpl, provider_auth::ProviderAuthRepoImpl, token::TokenRepoImpl},
        health::world::WorldRepoImpl,
    },
};

use super::{AppConfig, AppInfo};

pub struct AppService {
    pub health: HealthService,
    pub email_auth: EmailAuthService,
    pub google_auth: GoogleAuthService,
    pub token: TokenService,
}

impl AppService {
    pub fn new(
        info: &AppInfo,
        conf: &AppConfig,
        db: Arc<PgPool>,
        system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
        random_string_generator: Arc<dyn RandomStringGenerator + Send + Sync>,
    ) -> Self {
        Self {
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
                token_generator: random_string_generator.clone(),
                jwt_conf: conf.jwt.clone(),
                token_repo: Arc::new(TokenRepoImpl { db }),
            },
        }
    }
}
