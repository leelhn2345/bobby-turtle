mod app;
mod database;
mod environment;
mod stickers;

pub use app::*;
pub use database::*;
pub use environment::*;
pub use stickers::*;

use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub application: AppSettings,
    pub database: DatabaseSettings,
    pub stickers: Stickers,
}

pub fn get_settings(env: &Environment) -> Result<Settings, figment::Error> {
    let base_path = std::env::current_dir().expect("failed to determine current working directory");
    let config_dir = base_path.join("config");

    let env_filename = format!("{}.yaml", env.as_str());

    Figment::new()
        .merge(Yaml::file(config_dir.join("base.yaml")))
        .merge(Yaml::file(config_dir.join(env_filename)))
        .merge(Yaml::file(config_dir.join("stickers.yaml")))
        .merge(Env::prefixed("APP_").split("__"))
        .extract()
}
