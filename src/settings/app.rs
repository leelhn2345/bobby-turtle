use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
    pub public_url: String,
}
