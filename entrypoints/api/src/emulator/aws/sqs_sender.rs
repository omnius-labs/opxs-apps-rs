use std::sync::Arc;

use async_trait::async_trait;
use omnius_core_cloud::aws::sqs::SqsSender;
use tokio::sync::{Mutex as TokioMutex, mpsc};

pub struct SqsSenderEmulator {
    message_sender: mpsc::Sender<String>,
    pub message_receiver: Arc<TokioMutex<mpsc::Receiver<String>>>,
}

#[async_trait]
impl SqsSender for SqsSenderEmulator {
    async fn send_message(&self, message: &str) -> anyhow::Result<()> {
        self.message_sender.send(message.to_string()).await?;
        Ok(())
    }
}

impl SqsSenderEmulator {
    #[allow(unused)]
    pub fn new() -> Self {
        let (message_sender, message_receiver) = mpsc::channel::<String>(32);
        Self {
            message_sender,
            message_receiver: Arc::new(TokioMutex::new(message_receiver)),
        }
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn send_test() -> TestResult {
        let sqs_sender = SqsSenderEmulator::new();
        sqs_sender.send_message("test1").await?;
        sqs_sender.send_message("test2").await?;
        sqs_sender.send_message("test3").await?;

        let v: String = sqs_sender.message_receiver.lock().await.recv().await.unwrap();
        assert_eq!(v, "test1");

        let v: String = sqs_sender.message_receiver.lock().await.recv().await.unwrap();
        assert_eq!(v, "test2");

        Ok(())
    }
}
