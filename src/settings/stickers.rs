use figment::{
    providers::{Format, Yaml},
    Figment,
};
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
    pub fn new() -> Result<Self, figment::Error> {
        let base_path =
            std::env::current_dir().expect("failed to determine current working directory");
        let config_dir = base_path.join("config");

        Figment::new()
            .merge(Yaml::file(config_dir.join("stickers.yaml")))
            .extract()
    }
}
