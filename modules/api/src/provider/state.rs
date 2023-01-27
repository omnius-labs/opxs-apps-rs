use crate::domain::AppState;

use super::ReposStateProvider;

pub struct AppStateProvider;

impl AppStateProvider {
    pub fn provide() -> AppState {
        AppState {
            repos: ReposStateProvider::provide(),
        }
    }
}
