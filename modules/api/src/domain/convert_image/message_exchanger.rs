// use axum::extract::ws::{Message, WebSocket};
// use futures_util::stream::{SplitSink, SplitStream, StreamExt};

// use crate::shared::AppError;

// pub trait WebSocketMessageReceiver {}

// pub struct WebSocketMessageExchanger;

// impl WebSocketMessageExchanger {
//     pub async fn append(&self, ws: WebSocket) -> Result<(), AppError> {
//         let msg = ws.recv().await;
//         if msg.is_none() {
//             return Err(AppError::WebSocketHandshakeError(anyhow::anyhow!("message not found")));
//         }

//         match msg.unwrap()? {
//             Message::Text(text) => {
//                 if ws.send(Message::from(text)).await.is_err() {
//                     break;
//                 }
//             }
//             Message::Binary(text) => {
//                 return Err(AppError::WebSocketHandshakeError(anyhow::anyhow!("binary format is not supported")));
//             }
//             Message::Close(_) => {
//                 return Err(AppError::WebSocketHandshakeError(anyhow::anyhow!("closed")));
//             }
//             _ => {
//             }
//         }

//         Ok(())
//     }
// }
