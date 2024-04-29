use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt, layer::SubscriberExt};

use crate::settings::environment::Environment;

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
