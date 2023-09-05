pub mod auth;

use axum::{
    extract::State,
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Json, Router,
};
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use serde_json::Value;

use crate::shared::{AppError, AppState};

#[allow(unused)]
pub fn gen_service(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws", get(handle_web_socket_upgrade))
        .with_state(state.clone())
        .nest_service("/auth", auth::gen_service(state.clone()))
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200)
    )
)]
#[allow(unused)]
pub async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let ret = state.service.health.check().await?;
    Ok(Json(ret))
}

async fn handle_web_socket_upgrade(wsu: WebSocketUpgrade) -> Response {
    wsu.on_upgrade(handle_web_socket)
}

async fn handle_web_socket(ws: WebSocket) {
    let (sender, receiver) = ws.split();

    tokio::spawn(write(sender));
    tokio::spawn(read(receiver));
}

async fn read(_receiver: SplitStream<WebSocket>) {
    // ...
}

async fn write(_sender: SplitSink<WebSocket, Message>) {
    // ...
}
