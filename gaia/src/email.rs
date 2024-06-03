use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct EmailSettings {
    pub api_key: SecretString,
    pub api: String,
}
