use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub database_url: String,
}

impl AppConfig {
    pub fn load(path: &str) -> Self {
        let config = Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .unwrap();

        config.try_deserialize().unwrap()
    }
}
