use my_app::{get_environment, get_settings, Environment};
use start::start_app;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt};

mod jobs;
mod start;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    init_tracing(&env);
    Box::pin(start_app(settings, env)).await;
}

pub fn init_tracing(env: &Environment) {
    let env_level = match *env {
        Environment::Local => LevelFilter::DEBUG,
        Environment::Production => LevelFilter::INFO,
    };

    let target_filter = Targets::new().with_target("telebot", env_level);

    let format_layer = fmt::layer()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    let subscriber = tracing_subscriber::registry()
        .with(format_layer)
        .with(target_filter);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let env_name = env.as_str();
    tracing::info!("telebot app started in {env_name} environment!");
}
