#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use telebot::settings::environment::get_environment;
use telebot::settings::get_settings;
use telebot::settings::telemetry::init_tracing;
use telebot::startup::start_app;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    init_tracing(&env);
    tracing::info!(?env);
    Box::pin(start_app(settings, env)).await;
}
