use gaia::{environment::get_environment, get_connection_pool, get_settings, init_tracing};
use gardener::start_app;
use tokio::signal;
use turtle_bot::start_bot;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    let pool = get_connection_pool(&env, &settings.database).await;

    init_tracing(&env, vec![("gardener")]);

    let bot = tokio::spawn(start_bot(env, settings.clone(), pool.clone()));
    let app = tokio::spawn(start_app(settings, pool));

    tokio::select! {
        _ = signal::ctrl_c() => tracing::info!("ctrl-c received"),
        _o = app => tracing::info!("web server has shutdown."),
        _o = bot => tracing::info!("telegram bot app has shutdown."),
    }
    // start_app(settings, pool).await;
}
