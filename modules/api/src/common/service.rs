use std::sync::Arc;

use chrono::Utc;
use omnius_core_base::{clock::SystemClock, random_bytes::RandomBytesProvider};
use omnius_core_cloud::aws::sqs::SqsSender;
use sqlx::PgPool;

use crate::{
    domain::{
        auth::{
            email::{EmailAuthRepo, EmailAuthService},
            google::{GoogleAuthService, GoogleOAuth2Provider, ProviderAuthRepo},
            token::{TokenRepo, TokenService},
            user::{UserRepo, UserService},
        },
        health::{repo::WorldRepo, service::HealthService},
    },
    service::{Kdf, KdfAlgorithm},
};

use super::{AppConfig, AppInfo};

pub struct AppService {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
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
        send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
    ) -> Self {
        Self {
            system_clock: system_clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),

            send_email_sqs_sender: send_email_sqs_sender.clone(),

            health: HealthService {
                info: info.clone(),
                world_repo: Arc::new(WorldRepo { db: db.clone() }),
            },
            email_auth: EmailAuthService {
                auth_repo: Arc::new(EmailAuthRepo { db: db.clone() }),
                system_clock: system_clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.jwt.clone(),
                kdf: Kdf {
                    algorithm: KdfAlgorithm::Pbkdf2HmacSha256,
                    iterations: 1024,
                },
            },
            google_auth: GoogleAuthService {
                oauth2_provider: Arc::new(GoogleOAuth2Provider {}),
                auth_repo: Arc::new(ProviderAuthRepo { db: db.clone() }),
                auth_conf: conf.auth.clone(),
            },
            token: TokenService {
                system_clock: system_clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.jwt.clone(),
                token_repo: Arc::new(TokenRepo { db: db.clone() }),
            },
            user: UserService {
                user_repo: Arc::new(UserRepo { db }),
            },
        }
    }
}
