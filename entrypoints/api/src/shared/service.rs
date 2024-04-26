use std::sync::Arc;

use chrono::Utc;
use opxs_base::{AppConfig, AppInfo};
use opxs_image_convert::{ImageConvertJobCreator, ImageConvertJobRepository};
use sqlx::PgPool;

use core_base::{clock::Clock, random_bytes::RandomBytesProvider, tsid::TsidProvider};
use core_cloud::aws::{s3::S3Client, sqs::SqsSender};
use opxs_auth::{
    email::{EmailAuthRepo, EmailAuthService},
    provider::{GoogleAuthService, GoogleOAuth2ProviderImpl, ProviderAuthRepo},
    shared::kdf::{Kdf, KdfAlgorithm},
    token::{TokenRepo, TokenService},
    user::{UserRepo, UserService},
};
use opxs_email_send::{EmailSendJobCreator, EmailSendJobRepository};

use crate::service::health::{repo::WorldRepo, service::HealthService};

pub struct AppService {
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
    pub tsid_provider: Arc<dyn TsidProvider + Send + Sync>,

    pub email_send_job_creator: EmailSendJobCreator,
    pub image_convert_job_creator: ImageConvertJobCreator,

    pub health: HealthService,
    pub email_auth: EmailAuthService,
    pub google_auth: GoogleAuthService,
    pub token: TokenService,
    pub user: UserService,
}

impl AppService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        info: &AppInfo,
        conf: &AppConfig,
        db: Arc<PgPool>,
        clock: Arc<dyn Clock<Utc> + Send + Sync>,
        random_bytes_provider: Arc<dyn RandomBytesProvider + Send + Sync>,
        tsid_provider: Arc<dyn TsidProvider + Send + Sync>,
        send_email_sqs_sender: Arc<dyn SqsSender + Send + Sync>,
        image_convert_s3_client: Arc<dyn S3Client + Send + Sync>,
    ) -> Self {
        Self {
            clock: clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),
            tsid_provider: tsid_provider.clone(),

            email_send_job_creator: EmailSendJobCreator {
                email_send_job_repository: Arc::new(EmailSendJobRepository {
                    db: db.clone(),
                    clock: clock.clone(),
                }),
                send_email_sqs_sender: send_email_sqs_sender.clone(),
            },

            image_convert_job_creator: ImageConvertJobCreator {
                image_convert_job_repository: Arc::new(ImageConvertJobRepository {
                    db: db.clone(),
                    clock: clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                clock: clock.clone(),
                s3_client: image_convert_s3_client,
            },

            health: HealthService {
                info: info.clone(),
                world_repo: Arc::new(WorldRepo { db: db.clone() }),
            },
            email_auth: EmailAuthService {
                auth_repo: Arc::new(EmailAuthRepo {
                    db: db.clone(),
                    clock: clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                clock: clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.auth.jwt.clone(),
                kdf: Kdf {
                    algorithm: KdfAlgorithm::Pbkdf2HmacSha256,
                    iterations: 1024,
                },
            },
            google_auth: GoogleAuthService {
                oauth2_provider: Arc::new(GoogleOAuth2ProviderImpl {}),
                auth_repo: Arc::new(ProviderAuthRepo {
                    db: db.clone(),
                    clock: clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                auth_conf: conf.auth.clone(),
            },
            token: TokenService {
                clock: clock.clone(),
                random_bytes_provider: random_bytes_provider.clone(),
                jwt_conf: conf.auth.jwt.clone(),
                token_repo: Arc::new(TokenRepo {
                    db: db.clone(),
                    clock: clock.clone(),
                }),
            },
            user: UserService {
                user_repo: Arc::new(UserRepo { db }),
            },
        }
    }
}
