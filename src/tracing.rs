use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt,
    layer::SubscriberExt,
    registry, EnvFilter, Layer,
};

use crate::settings::Environment;

pub fn init_tracing(env: Environment) {
    let env_layer = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()));

    let emit_bunyan = env == Environment::Production;
    let bunyan_json_layer = JsonStorageLayer.with_filter(filter_fn(move |_| emit_bunyan));
    let bunyan_formatting_layer = BunyanFormattingLayer::new("telebot".into(), std::io::stdout)
        .with_filter(filter_fn(move |_| emit_bunyan));

    let emit_pretty = env == Environment::Local;
    let pretty_formatting_layer = fmt::layer().with_filter(filter_fn(move |_| emit_pretty));

    let subscriber = registry()
        .with(env_layer)
        .with(bunyan_json_layer)
        .with(bunyan_formatting_layer)
        .with(pretty_formatting_layer);

    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");
}
