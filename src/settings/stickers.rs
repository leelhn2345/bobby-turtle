use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Stickers {
    pub kiss: String,
    pub hello: String,
    pub hug: String,
    pub coming_soon: String,
    pub sad: String,
    pub party_animals: Vec<String>,
    pub sleep: String,
    pub lame: String,
    pub angry: String,
    pub devil: String,
    pub flower: String,
    pub love: String,
    pub laugh: String,
}

impl Stickers {
    pub fn new() -> Result<Self, ConfigError> {
        let base_path =
            std::env::current_dir().expect("failed to determine current working directory");
        let config_dir = base_path.join("config");

        let util = config::Config::builder()
            .add_source(config::File::from(config_dir.join("stickers.yaml")))
            .build()?;

        util.try_deserialize::<Stickers>()
    }
}
