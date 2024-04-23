use serde::Deserialize;

/// The stickers used in this app.
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
