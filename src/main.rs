mod bot;
mod handlers;
mod routes;
mod settings;
mod stickers;
mod types;
mod web;

use bot::start_bot;
use settings::get_settings;
use teloxide::Bot;
use web::setup_axum_webhook;

#[tokio::main]
async fn main() {
    let settings = get_settings().expect("failed to read settings");
    let tele_bot = Bot::from_env();

    let listener = setup_axum_webhook(&settings, tele_bot.clone()).await;

    start_bot(tele_bot, listener, settings).await;
}
