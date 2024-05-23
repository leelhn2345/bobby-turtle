mod bot;

use anyhow::Context;
use async_openai::Client;
use bot::bot_handler;
use gaia::{app::AppSettings, environment::Environment, Settings};
use sqlx::PgPool;
use teloxide::{
    dispatching::Dispatcher,
    dptree,
    error_handlers::LoggingErrorHandler,
    update_listeners::webhooks::{self, Options},
    Bot,
};

pub async fn start_bot(env: Environment, settings: Settings, pool: PgPool) {
    let tele_bot = Bot::from_env();
    let chatgpt = Client::new();

    let options = get_webhook_options(&settings.application, &env);

    let listener = webhooks::axum(tele_bot.clone(), options)
        .await
        .map_err(|e| tracing::error!("{e:#?}"))
        .expect("unable to get listener");

    let handler = bot_handler();

    Dispatcher::builder(tele_bot, handler)
        .dependencies(dptree::deps![
            settings.stickers,
            chatgpt,
            pool // InMemStorage::<ChatState>::new(),
                 // InMemStorage::<CallbackPage>::new(),
                 // sched
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

fn get_webhook_options(settings: &AppSettings, env: &Environment) -> Options {
    let address = format!("{}:{}", settings.host, settings.bot_port)
        .parse()
        .context(format!("{}:{}", settings.host, settings.bot_port,))
        .expect("unable to parse into address url");

    let url = format!("{}/webhook", settings.public_url)
        .parse()
        .context(settings.public_url.to_string())
        .expect("unable to parse into webhook url");

    let mut options = webhooks::Options::new(address, url);

    if *env == Environment::Local {
        options = options.drop_pending_updates();
        tracing::info!("bot started in http://localhost:{}", settings.bot_port);
    }

    options
}
