use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;

use core_base::{clock::SystemClock, random_bytes::RandomBytesProvider, tsid::TsidProvider};
use core_cloud::aws::sqs::SqsSender;
use opxs_auth::{
    email::{EmailAuthRepo, EmailAuthService},
    provider::{GoogleAuthService, GoogleOAuth2ProviderImpl, ProviderAuthRepo},
    shared::kdf::{Kdf, KdfAlgorithm},
    token::{TokenRepo, TokenService},
    user::{UserRepo, UserService},
};
use opxs_email_send::{EmailSendJobCreator, EmailSendJobRepository};

use crate::domain::health::{repo::WorldRepo, service::HealthService};

use super::{config::AppConfig, info::AppInfo};

pub struct AppService {
    pub system_clock: Arc<dyn SystemClock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub tsid_provider: Arc<dyn TsidProvider + Send + Sync>,

    pub email_send_job_creator: EmailSendJobCreator,

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
        tsid_provider: Arc<dyn TsidProvider + Send + Sync>,
        send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
    ) -> Self {
        Self {
            system_clock: system_clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),
            tsid_provider: tsid_provider.clone(),

            email_send_job_creator: EmailSendJobCreator {
                email_send_job_repository: Arc::new(EmailSendJobRepository {
                    db: db.clone(),
                    system_clock: system_clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                send_email_sqs_sender: send_email_sqs_sender.clone(),
            },

            health: HealthService {
                info: info.clone(),
                world_repo: Arc::new(WorldRepo { db: db.clone() }),
            },
            email_auth: EmailAuthService {
                auth_repo: Arc::new(EmailAuthRepo {
                    db: db.clone(),
                    system_clock: system_clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                system_clock: system_clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.jwt.clone(),
                kdf: Kdf {
                    algorithm: KdfAlgorithm::Pbkdf2HmacSha256,
                    iterations: 1024,
                },
            },
            google_auth: GoogleAuthService {
                oauth2_provider: Arc::new(GoogleOAuth2ProviderImpl {}),
                auth_repo: Arc::new(ProviderAuthRepo {
                    db: db.clone(),
                    system_clock: system_clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
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
