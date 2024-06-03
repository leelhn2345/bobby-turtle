pub mod app;
pub mod database;
pub mod email;
pub mod environment;
pub mod stickers;

use app::AppSettings;
pub use database::get_connection_pool;
use database::DatabaseSettings;
use email::EmailSettings;
use environment::Environment;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::Deserialize;
use stickers::Stickers;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt, layer::SubscriberExt};

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub application: AppSettings,
    pub email: EmailSettings,
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

/// * `env`: `local` or `production`
/// * `targets`: vector of crates whose trace we are interested in.
pub fn init_tracing(env: &Environment, targets: Vec<&str>) {
    let env_level = match *env {
        Environment::Local => LevelFilter::DEBUG,
        Environment::Production => LevelFilter::INFO,
    };

    let targets_with_level: Vec<(&str, LevelFilter)> =
        targets.into_iter().map(|s| (s, env_level)).collect();

    let target_filter = Targets::new().with_targets(targets_with_level);

    let format_layer = fmt::layer()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    let subscriber = tracing_subscriber::registry()
        .with(format_layer)
        .with(target_filter);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");
}
