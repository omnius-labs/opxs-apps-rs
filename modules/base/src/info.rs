use std::{env, fmt};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunMode {
    Local,
    Dev,
}

impl RunMode {
    pub fn from_env() -> Result<Self> {
        let mode = env::var("RUN_MODE")?;
        match mode.as_str() {
            "local" => Ok(RunMode::Local),
            "dev" => Ok(RunMode::Dev),
            _ => Err(Error::builder()
                .kind(ErrorKind::InvalidFormat)
                .message(format!("invalid RUN_MODE: {}", mode))
                .build()),
        }
    }
}

impl fmt::Display for RunMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunMode::Local => write!(f, "local"),
            RunMode::Dev => write!(f, "dev"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub app_name: String,
    pub mode: RunMode,
    pub git_tag: String,
}

impl AppInfo {
    pub fn new(app_name: &str, mode: RunMode) -> Result<Self> {
        let git_tag = option_env!("GIT_TAG").unwrap_or("unknown").to_string();

        Ok(Self {
            app_name: app_name.to_string(),
            mode,
            git_tag,
        })
    }
}

impl fmt::Display for AppInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mode: {:?},", self.mode)?;
        Ok(())
    }
}
