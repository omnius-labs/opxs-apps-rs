use std::sync::Arc;

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use chrono::Utc;
use futures::FutureExt;
use parking_lot::Mutex;
use sqlx::PgPool;
use tempfile::{tempdir, TempDir};
use tokio::{sync::Mutex as TokioMutex, task::JoinHandle};
use tracing::error;

use omnius_core_base::{
    clock::{Clock, ClockUtc},
    random_bytes::{RandomBytesProvider, RandomBytesProviderImpl},
    terminable::Terminable,
    tsid::{TsidProvider, TsidProviderImpl},
};
use omnius_core_cloud::aws::{s3::S3ClientImpl, sqs::SqsSenderImpl};

use omnius_opxs_auth::{
    email::{EmailAuthRepo, EmailAuthService},
    provider::{GoogleAuthService, GoogleOAuth2ProviderImpl, ProviderAuthRepo},
    shared::kdf::{Kdf, KdfAlgorithm},
    token::{TokenRepo, TokenService},
    user::{UserRepo, UserService},
};
use omnius_opxs_base::{AppConfig, AppInfo};
use omnius_opxs_email_send::{EmailSendExecutor, EmailSendJobBatchSqsMessage, EmailSendJobCreator, EmailSendJobRepository};
use omnius_opxs_file_convert::{ImageConvertExecutor, ImageConvertJobCreator, ImageConvertJobRepository, ImageConverterImpl};

use crate::{
    emulator::aws::{S3ClientEmulator, S3ClientEmulatorOption, SesSenderEmulator, SqsSenderEmulator},
    service::health::{repo::WorldRepo, service::HealthService},
};

pub struct AppService {
    pub clock: Arc<dyn Clock<Utc> + Send + Sync>,
    pub random_bytes_provider: Arc<Mutex<dyn RandomBytesProvider + Send + Sync>>,
    pub tsid_provider: Arc<Mutex<dyn TsidProvider + Send + Sync>>,

    pub email_send_job_creator: EmailSendJobCreator,
    pub image_convert_job_creator: ImageConvertJobCreator,

    pub health: HealthService,
    pub email_auth: EmailAuthService,
    pub google_auth: GoogleAuthService,
    pub token: TokenService,
    pub user: UserService,

    #[allow(clippy::type_complexity)]
    terminables: Box<TokioMutex<Option<Vec<Arc<dyn Terminable + Send + Sync>>>>>,
    join_handles: Box<TokioMutex<Option<Vec<JoinHandle<()>>>>>,

    #[allow(unused)]
    temp_dirs: Vec<TempDir>,
}

impl AppService {
    pub async fn new_for_cloud(info: &AppInfo, conf: &AppConfig, db: Arc<PgPool>) -> anyhow::Result<Self> {
        let clock = Arc::new(ClockUtc);
        let random_bytes_provider = Arc::new(Mutex::new(RandomBytesProviderImpl::new()));
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));

        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let send_email_sqs_sender = Arc::new(SqsSenderImpl {
            client: aws_sdk_sqs::Client::new(&sdk_config),
            queue_url: "opxs-batch-email-send-sqs".to_string(),
            delay_seconds: None,
        });
        let image_convert_s3_client = Arc::new(S3ClientImpl {
            client: aws_sdk_s3::Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await),
            bucket: conf
                .image
                .convert
                .s3
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("s3 config is not found"))?
                .bucket
                .clone(),
        });

        Ok(Self {
            clock: clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),
            tsid_provider: tsid_provider.clone(),

            email_send_job_creator: EmailSendJobCreator {
                email_send_job_repository: Arc::new(EmailSendJobRepository {
                    db: db.clone(),
                    clock: clock.clone(),
                }),
                sqs_sender: send_email_sqs_sender.clone(),
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

            terminables: Box::new(TokioMutex::new(None)),
            join_handles: Box::new(TokioMutex::new(None)),
            temp_dirs: vec![],
        })
    }

    pub async fn new_for_local(info: &AppInfo, conf: &AppConfig, db: Arc<PgPool>) -> anyhow::Result<Self> {
        let clock = Arc::new(ClockUtc);
        let random_bytes_provider = Arc::new(Mutex::new(RandomBytesProviderImpl::new()));
        let tsid_provider = Arc::new(Mutex::new(TsidProviderImpl::new(ClockUtc, RandomBytesProviderImpl::new(), 16)));

        let mut terminables: Vec<Arc<dyn Terminable + Send + Sync>> = Vec::new();
        let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
        let mut temp_dirs: Vec<TempDir> = Vec::new();

        let email_send_job_creator = {
            let sqs_sender = Arc::new(SqsSenderEmulator::new());
            let job_creator = EmailSendJobCreator {
                email_send_job_repository: Arc::new(EmailSendJobRepository {
                    db: db.clone(),
                    clock: clock.clone(),
                }),
                sqs_sender: sqs_sender.clone(),
            };

            let db = db.clone();
            let clock = clock.clone();
            let message_receiver = sqs_sender.message_receiver.clone();

            let join_handle: JoinHandle<()> = tokio::spawn(async move {
                let ses_sender = Arc::new(SesSenderEmulator::new());
                let executor = EmailSendExecutor {
                    email_send_job_repository: Arc::new(EmailSendJobRepository { db, clock }),
                    ses_sender,
                };

                loop {
                    if let Some(message) = message_receiver.lock().await.recv().await {
                        let message = match serde_json::from_str::<EmailSendJobBatchSqsMessage>(&message) {
                            Ok(message) => message,
                            _ => {
                                error!("email send sqs message parse failed");
                                continue;
                            }
                        };

                        if let Err(err) = executor.execute(&[message]).await {
                            error!("email send execute error: {:?}", err);
                        }
                    }
                }
            });
            join_handles.push(join_handle);

            job_creator
        };

        let image_convert_job_creator = {
            let working_dir = tempdir()?;

            let option = S3ClientEmulatorOption {
                base_url: "http://localhost:40000".parse()?,
                listen_addr: "0.0.0.0:40000".parse()?,
                working_dir: working_dir.path().to_path_buf(),
            };
            let s3_client = Arc::new(S3ClientEmulator::new(option)?);
            let job_creator = ImageConvertJobCreator {
                image_convert_job_repository: Arc::new(ImageConvertJobRepository {
                    db: db.clone(),
                    clock: clock.clone(),
                    tsid_provider: tsid_provider.clone(),
                }),
                clock: clock.clone(),
                s3_client: s3_client.clone(),
            };

            terminables.push(s3_client.clone());

            let db = db.clone();
            let clock = clock.clone();
            let tsid_provider = tsid_provider.clone();
            let s3_client = s3_client.clone();
            let put_event_receiver = s3_client.put_event_receiver.clone();

            let join_handle: JoinHandle<()> = tokio::spawn(async move {
                let executor = ImageConvertExecutor {
                    image_converter: Arc::new(ImageConverterImpl),
                    image_convert_job_repository: Arc::new(ImageConvertJobRepository { db, clock, tsid_provider }),
                    s3_client,
                };

                loop {
                    if let Some(job_id) = put_event_receiver.lock().await.recv().await {
                        if let Err(err) = executor.execute(&[job_id]).await {
                            error!("image convert error: {:?}", err);
                        }
                    }
                }
            });
            join_handles.push(join_handle);
            temp_dirs.push(working_dir);

            job_creator
        };

        Ok(Self {
            clock: clock.clone(),
            random_bytes_provider: random_bytes_provider.clone(),
            tsid_provider: tsid_provider.clone(),

            email_send_job_creator,

            image_convert_job_creator,

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

            terminables: Box::new(TokioMutex::new(Some(terminables))),
            join_handles: Box::new(TokioMutex::new(Some(join_handles))),
            temp_dirs,
        })
    }
}

#[async_trait]
impl Terminable for AppService {
    async fn terminate(&self) -> anyhow::Result<()> {
        if let Some(ts) = self.terminables.lock().await.take() {
            for t in ts {
                t.terminate().await?;
            }
        }

        if let Some(js) = self.join_handles.lock().await.take() {
            for j in js {
                j.abort();
                j.fuse().await?;
            }
        }

        Ok(())
    }
}
