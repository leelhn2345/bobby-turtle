#![warn(clippy::pedantic)]

use telebot::{startup::start_app, telemetry::init_tracing};

#[tokio::main]
async fn main() {
    init_tracing();
    start_app().await;
}
