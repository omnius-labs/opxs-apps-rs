use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;

use crate::ImageConverter;

pub struct ImageConverterMock {
    pub convert_inputs: Arc<Mutex<Vec<ConvertInput>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertInput {
    source: String,
    target: String,
}

#[async_trait]
impl ImageConverter for ImageConverterMock {
    async fn convert(&self, source: &str, target: &str) -> anyhow::Result<()> {
        self.convert_inputs.lock().push(ConvertInput {
            source: source.to_string(),
            target: target.to_string(),
        });

        Ok(())
    }
}

impl ImageConverterMock {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            convert_inputs: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Default for ImageConverterMock {
    fn default() -> Self {
        Self::new()
    }
}
