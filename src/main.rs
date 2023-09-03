mod bot;
mod routes;
mod settings;
mod webhook;

use bot::start_bot;
use settings::get_settings;
use teloxide::Bot;
use webhook::setup_axum_webhook;

#[tokio::main]
async fn main() {
    let settings = get_settings().expect("failed to read settings");
    let bot = Bot::from_env();

    let listener = setup_axum_webhook(settings, bot.clone()).await;

    start_bot(bot, listener).await;
}
