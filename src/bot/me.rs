use once_cell::sync::Lazy;
use std::sync::OnceLock;
use teloxide::{requests::Requester, types::Me, Bot};
use tracing::instrument;
pub static BOT_ME: Lazy<&Me> = Lazy::new(|| BOT_DETAILS.get().unwrap());

static BOT_DETAILS: OnceLock<Me> = OnceLock::new();

#[instrument(name = "set up bot details", skip_all)]
pub async fn setup_me(bot: &Bot) {
    let me = bot.get_me().await.expect("cannot get details about bot");

    BOT_DETAILS.set(me).unwrap();
    tracing::info!("success")
}
