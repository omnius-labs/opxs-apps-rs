pub mod convert;

use axum::Router;

use crate::shared::state::AppState;

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .nest_service("/convert", convert::gen_service(state.clone()))
        .with_state(state)
}
