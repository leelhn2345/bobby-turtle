use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub bot_port: u16,
    pub web_port: u16,
    pub host: String,
    pub public_url: String,
}
