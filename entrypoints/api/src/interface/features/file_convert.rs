pub mod image;

use axum::Router;

use crate::shared::state::AppState;

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new().nest_service("/image", image::gen_service(state.clone())).with_state(state)
}
