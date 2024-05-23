use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub bot_port: u16,
    pub web_port: u16,
    pub host: String,
    pub public_url: String,
}
