use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub mode: String,
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
            mode,
            git_semver,
            git_sha,
            build_timestamp,
        })
    }
}
