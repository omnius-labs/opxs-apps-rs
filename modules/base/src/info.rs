use std::{env, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunMode {
    Local,
    Dev,
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
    pub mode: RunMode,
    pub git_tag: String,
}

impl AppInfo {
    pub fn new() -> anyhow::Result<Self> {
        let mode = env::var("RUN_MODE")?;
        let git_tag = option_env!("GIT_TAG").unwrap_or("unknown").to_string();

        Ok(Self {
            mode: match mode.as_str() {
                "local" => RunMode::Local,
                "dev" => RunMode::Dev,
                _ => anyhow::bail!("invalid RUN_MODE: {}", mode),
            },
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
