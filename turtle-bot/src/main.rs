use gaia::{environment::get_environment, get_connection_pool, get_settings, init_tracing};
use teloxide::Bot;
use turtle_bot::start_bot;

#[tokio::main]
async fn main() {
    let env = get_environment();
    let settings = get_settings(&env).expect("failed to parse settings");
    let pool = get_connection_pool(&env, &settings.database).await;
    let bot = Bot::from_env();

    init_tracing(&env, vec![("turtle_bot")]);

    Box::pin(start_bot(bot, env, settings, pool)).await;
}
