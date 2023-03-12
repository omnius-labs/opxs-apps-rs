use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    domain::{auth::service::auth::AuthService, health::service::HealthService},
    infra::auth::{RefreshTokenRepoImpl, UserRepoImpl},
};

use super::AppConfig;

#[derive(Clone)]
pub struct AppService {
    pub health: HealthService,
    pub auth: AuthService,
}

impl AppService {
    pub fn new(db: &Arc<PgPool>, conf: &AppConfig) -> Self {
        let user_repo = Arc::new(UserRepoImpl { db: db.clone() });
        let refresh_token_repo = Arc::new(RefreshTokenRepoImpl { db: db.clone() });

        Self {
            health: HealthService {},
            auth: AuthService {
                user_repo,
                refresh_token_repo,
                jwt_conf: conf.jwt.clone(),
            },
        }
    }
}
