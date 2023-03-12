use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::ToSchema;

use crate::{
    infra::auth::PostgresAuthRepo,
    shared::{AppError, AppState},
    usecase::auth::AuthUseCase,
};

#[allow(unused)]
pub fn auth(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterParam,
    responses(
        (status = 200, description = "", body = Registered)
    )
)]
#[allow(unused)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterParam>,
) -> Result<Json<Value>, AppError> {
    if req.name.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    let auth_repo = Arc::new(PostgresAuthRepo { db: state.db });
    let auth_usecase = AuthUseCase { auth_repo };

    auth_usecase
        .register(&req.name, &req.email, &req.password)
        .await?;

    Ok(Json(json!({"message": "registered successfully"})))
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterParam {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct Registered {
    id: i64,
}
