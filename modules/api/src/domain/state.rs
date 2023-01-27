use std::sync::Arc;

use super::UserRepo;

#[derive(Clone)]
pub struct AppState {
    pub repos: ReposState,
}

#[derive(Clone)]
pub struct ReposState {
    pub user: Arc<dyn UserRepo>,
}
