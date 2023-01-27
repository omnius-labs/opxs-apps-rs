use std::sync::Arc;

use crate::{
    domain::{ReposState, UserRepo},
    infra::UserRepoImpl,
};

pub struct ReposStateProvider;

impl ReposStateProvider {
    pub fn provide() -> ReposState {
        ReposState {
            user: UserRepoProvider::provide(),
        }
    }
}

pub struct UserRepoProvider;

impl UserRepoProvider {
    pub fn provide() -> Arc<dyn UserRepo> {
        Arc::new(UserRepoImpl {})
    }
}
