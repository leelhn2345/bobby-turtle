use turtle_bot::{
    bot::start_bot,
    settings::{get_environment, get_settings},
    telemetry::init_tracing,
};

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings().expect("failed to read settings");
    init_tracing("turtle_bot=info".into());

    tracing::info!("starting app~");

    start_bot(settings, env).await;
}
