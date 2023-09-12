mod bot;
mod handlers;
mod routes;
mod settings;
mod stickers;
mod tracing;
mod types;
mod web;

use bot::start_bot;
use settings::{get_environment, get_settings};
use teloxide::Bot;
use tracing::init_tracing;
use web::setup_axum_webhook;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to read settings");
    init_tracing(env, "turtle_bot=info".into());
    let tele_bot = Bot::from_env();

    let listener = setup_axum_webhook(&settings, tele_bot.clone()).await;

    start_bot(tele_bot, listener, settings).await;
}
