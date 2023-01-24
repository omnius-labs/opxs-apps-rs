// use config::Config;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    postgres_: Vec<String>,
}

/*
impl AppConfig {
    pub fn load() -> Self {
        std::env::set_var("APP_LIST", "Hello World");

        let config = Config::builder()
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()
            .unwrap();

        let app: AppConfig = config.try_deserialize().unwrap();
    }
}
*/
