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
}

impl AppInfo {
    pub fn new() -> anyhow::Result<Self> {
        let mode = env::var("RUN_MODE")?;

        Ok(Self {
            mode: match mode.as_str() {
                "local" => RunMode::Local,
                "dev" => RunMode::Dev,
                _ => anyhow::bail!("invalid RUN_MODE: {}", mode),
            },
        })
    }
}

impl fmt::Display for AppInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mode: {:?},", self.mode)?;
        Ok(())
    }
}
