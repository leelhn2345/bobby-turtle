mod app;
pub mod environment;
pub mod stickers;
pub mod telemetry;

use app::AppSettings;
use config::ConfigError;
use serde::Deserialize;

use crate::settings::environment::Environment;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub application: AppSettings,
}

pub fn get_settings(env: &Environment) -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("failed to determine current working directory");
    let config_dir = base_path.join("config");

    let env_filename = format!("{}.yaml", env.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base.yaml")))
        .add_source(config::File::from(config_dir.join(env_filename)))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
