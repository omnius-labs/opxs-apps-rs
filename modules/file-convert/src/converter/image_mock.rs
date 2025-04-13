use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use parking_lot::Mutex;

use crate::{FileConvertImageInputFileType, FileConvertImageOutputFileType, prelude::*};

use super::ImageConverter;

pub struct ImageConverterMock {
    pub convert_inputs: Arc<Mutex<Vec<ConvertInput>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertInput {
    in_path: PathBuf,
    in_type: FileConvertImageInputFileType,
    out_path: PathBuf,
    out_type: FileConvertImageOutputFileType,
}

#[async_trait]
impl ImageConverter for ImageConverterMock {
    async fn convert(
        &self,
        in_path: &Path,
        in_type: &FileConvertImageInputFileType,
        out_path: &Path,
        out_type: &FileConvertImageOutputFileType,
    ) -> Result<()> {
        self.convert_inputs.lock().push(ConvertInput {
            in_path: in_path.to_path_buf(),
            in_type: in_type.clone(),
            out_path: out_path.to_path_buf(),
            out_type: out_type.clone(),
        });

        Ok(())
    }
}

impl ImageConverterMock {
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
