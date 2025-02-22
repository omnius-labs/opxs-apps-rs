pub mod email;
pub mod google;
pub mod token;

use axum::{Json, Router, routing::get};

use omnius_opxs_auth::shared::model::User;
use omnius_opxs_base::AppError;

use crate::shared::state::AppState;

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/me", get(me))
        .nest_service("/email", email::gen_service(state.clone()))
        .nest_service("/google", google::gen_service(state.clone()))
        .nest_service("/token", token::gen_service(state.clone()))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200)
    ),
    security(
        ("bearer_token" = [])
    )
)]
pub async fn me(user: User) -> Result<Json<User>, AppError> {
    Ok(Json(user))
}
