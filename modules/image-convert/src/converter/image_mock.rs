use std::cell::RefCell;

use async_trait::async_trait;

use crate::ImageConverter;

pub struct ImageConverterMock {
    pub convert_inputs: RefCell<Vec<ConvertInput>>,
}

unsafe impl Sync for ImageConverterMock {}
unsafe impl Send for ImageConverterMock {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertInput {
    source: String,
    target: String,
}

#[async_trait]
impl ImageConverter for ImageConverterMock {
    async fn convert(&self, source: &str, target: &str) -> anyhow::Result<()> {
        self.convert_inputs.borrow_mut().push(ConvertInput {
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
            convert_inputs: RefCell::new(vec![]),
        }
    }
}

impl Default for ImageConverterMock {
    fn default() -> Self {
        Self::new()
    }
}
