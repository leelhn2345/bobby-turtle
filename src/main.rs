use telebot::settings::environment::get_environment;
use telebot::settings::get_settings;
use telebot::settings::telemetry::init_tracing;
use telebot::start::start_app;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    init_tracing(&env);
    Box::pin(start_app(settings, env)).await;
}
