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
    pub git_semver: String,
    pub git_sha: String,
    pub build_timestamp: String,
}

impl AppInfo {
    pub fn new() -> anyhow::Result<Self> {
        let mode = env::var("RUN_MODE")?;
        let git_semver = env!("VERGEN_GIT_SEMVER").to_string();
        let git_sha = env!("VERGEN_GIT_SHA").to_string();
        let build_timestamp = env!("VERGEN_BUILD_TIMESTAMP").to_string();

        Ok(Self {
            mode: match mode.as_str() {
                "local" => RunMode::Local,
                "dev" => RunMode::Dev,
                _ => anyhow::bail!("invalid RUN_MODE: {}", mode),
            },
            git_semver,
            git_sha,
            build_timestamp,
        })
    }
}

impl fmt::Display for AppInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mode: {:?},", self.mode)?;
        write!(f, "git_semver: {},", self.git_semver)?;
        write!(f, "git_sha: {},", self.git_sha)?;
        write!(f, "build_timestamp: {}", self.build_timestamp)?;
        Ok(())
    }
}
