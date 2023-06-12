use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    domain::{
        auth::service::auth::{AuthService, Kdf, KdfAlgorithm},
        health::service::HealthService,
    },
    infra::auth::{RefreshTokenRepoImpl, UserRepoImpl},
};

use super::{AppConfig, AppInfo};

#[derive(Clone)]
pub struct AppService {
    pub health: HealthService,
    pub auth: AuthService,
}

impl AppService {
    pub fn new(info: &AppInfo, conf: &AppConfig, db: &Arc<PgPool>) -> Self {
        let kdf = Kdf::new(KdfAlgorithm::Pbkdf2HmacSha256, 10_000);
        let user_repo = Arc::new(UserRepoImpl { db: db.clone() });
        let refresh_token_repo = Arc::new(RefreshTokenRepoImpl { db: db.clone() });

        Self {
            health: HealthService { info: info.clone() },
            auth: AuthService {
                kdf,
                user_repo,
                refresh_token_repo,
                jwt_conf: conf.jwt.clone(),
            },
        }
    }
}
