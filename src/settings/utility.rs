use config::ConfigError;
use serde::Deserialize;

use super::stickers::Stickers;

#[derive(Deserialize)]
pub struct Utility {
    pub stickers: Stickers,
}

impl Utility {
    pub fn new() -> Result<Self, ConfigError> {
        let base_path =
            std::env::current_dir().expect("failed to determine current working directory");
        let config_dir = base_path.join("config");

        let util = config::Config::builder()
            .add_source(config::File::from(config_dir.join("stickers.yaml")))
            .build()?;

        util.try_deserialize::<Utility>()
    }
}
