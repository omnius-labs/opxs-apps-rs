use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

use omnius_core_cloud::aws::ses::SesSender;

pub struct SesSenderEmulator {}

#[async_trait]
impl SesSender for SesSenderEmulator {
    async fn send_mail_simple_text(
        &self,
        to_address: &str,
        from_address: &str,
        subject: &str,
        text_body: &str,
    ) -> anyhow::Result<String> {
        info!(
            target: "send_mail_simple_text",
            to_address,
            from_address,
            subject,
            text_body
        );
        let message_id = "emulator_".to_string() + Uuid::new_v4().simple().to_string().as_str();
        Ok(message_id)
    }
}

impl SesSenderEmulator {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;
    use tracing_subscriber::EnvFilter;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn send_test() -> TestResult {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=off"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .json()
            .init();

        let ses_sender = SesSenderEmulator {};
        ses_sender
            .send_mail_simple_text("to_address", "from_address", "subject", "text_body")
            .await?;

        Ok(())
    }
}
