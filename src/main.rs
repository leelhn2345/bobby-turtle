#![warn(clippy::pedantic)]

use telebot::settings::environment::get_environment;
use telebot::settings::telemetry::init_tracing;
use telebot::startup::start_app;

#[tokio::main]
async fn main() {
    let env = get_environment();
    init_tracing(&env);

    let env_name = &env.as_str();
    tracing::info!("telebot app started in {env_name} environment!");

    start_app().await;
}
