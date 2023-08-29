use polarrbear_bot::bot::start_bot;
use polarrbear_bot::settings::get_settings;
use polarrbear_bot::webhook::setup_axum_webhook;
use teloxide::Bot;

#[tokio::main]
async fn main() {
    let settings = get_settings().expect("failed to read settings");
    let bot = Bot::from_env();

    let listener = setup_axum_webhook(settings, bot.clone()).await;

    start_bot(bot, listener).await;
}
