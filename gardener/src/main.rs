use gaia::{environment::get_environment, get_connection_pool, get_settings, init_tracing};
use gardener::start_app;
use teloxide::Bot;
use tokio::signal;
use turtle_bot::start_bot;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    let pool = get_connection_pool(&env, &settings.database).await;

    init_tracing(&env, vec!["gardener", "turtle_bot"]);

    let bot = Bot::from_env();

    let bot_app = tokio::spawn(start_bot(bot.clone(), env, settings.clone(), pool.clone()));
    let web_app = tokio::spawn(start_app(settings, pool, bot));

    tokio::select! {
        _ = signal::ctrl_c() => tracing::info!("ctrl-c received"),
        _o = web_app => tracing::info!("web server has shutdown."),
        _o = bot_app => tracing::info!("telegram bot app has shutdown."),
    }
}
