use config::ConfigError;

mod application;
mod database;
mod environment;
mod stickers;

use application::*;
pub use environment::*;
use serde::Deserialize;
use stickers::*;
#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub stickers: Stickers,
    // pub users: Users,
    // pub database: DatabaseSettings,
}

pub fn get_settings(env: &Environment) -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("failed to determine current directory");
    let config_directory = base_path.join("config");

    let environment_filename = format!("{}.yaml", env.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(config_directory.join("base.yaml")))
        .add_source(config::File::from(
            config_directory.join(environment_filename),
        ))
        .add_source(config::File::from(config_directory.join("stickers.yaml")))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize()
}
