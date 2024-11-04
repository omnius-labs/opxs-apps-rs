use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use parking_lot::Mutex;

use crate::ImageConverter;

pub struct ImageConverterMock {
    pub convert_inputs: Arc<Mutex<Vec<ConvertInput>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertInput {
    source: PathBuf,
    target: PathBuf,
}

#[async_trait]
impl ImageConverter for ImageConverterMock {
    async fn convert(&self, source: &Path, target: &Path) -> anyhow::Result<()> {
        self.convert_inputs.lock().push(ConvertInput {
            source: source.to_path_buf(),
            target: target.to_path_buf(),
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
