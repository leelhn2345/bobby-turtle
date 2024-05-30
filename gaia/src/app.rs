use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub bot_port: u16,
    pub web_port: u16,
    pub host: String,
    pub public_url: String,
    pub request_origin: SecretString,
    pub cookie_key: SecretString,
}
