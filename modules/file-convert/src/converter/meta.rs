use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{FileConvertMetaInputFileType, FileConvertMetaOutputFileType, prelude::*};

#[async_trait]
pub trait MetaConverter {
    async fn convert(
        &self,
        in_path: &Path,
        in_type: &FileConvertMetaInputFileType,
        out_path: &Path,
        out_type: &FileConvertMetaOutputFileType,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct MetaConverterImpl;

#[async_trait]
impl MetaConverter for MetaConverterImpl {
    async fn convert(
        &self,
        _in_path: &Path,
        _in_type: &FileConvertMetaInputFileType,
        _out_path: &Path,
        _out_type: &FileConvertMetaOutputFileType,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct MetaConverterOption {
    pub in_path: String,
    pub in_type: FileConvertMetaInputFileType,
    pub out_path: String,
    pub out_type: FileConvertMetaOutputFileType,
}

#[cfg(test)]
mod tests {
    #[ignore]
    #[tokio::test]
    async fn simple_test() {}
}
