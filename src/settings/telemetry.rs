use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt};

use crate::settings::environment::Environment;

pub fn init_tracing(env: &Environment) {
    let env_level = match *env {
        Environment::Local => format!("telebot={}", LevelFilter::DEBUG)
            .parse()
            .expect("unable to set tracing level for local runtime"),
        Environment::Production => format!("telebot={}", LevelFilter::INFO)
            .parse()
            .expect("unable to set tracing level for production runtime"),
    };

    let env_layer = EnvFilter::from_default_env().add_directive(env_level);

    let format_layer = fmt::layer().without_time().with_target(false);

    let subscriber = tracing_subscriber::registry()
        .with(env_layer)
        .with(format_layer);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set tracing subscriber");

    let env_name = &env.as_str();
    tracing::info!("telebot app started in {env_name} environment!");
}
