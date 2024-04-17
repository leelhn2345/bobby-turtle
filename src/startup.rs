use std::convert::Infallible;

use anyhow::Context;
use async_openai::{config::OpenAIConfig, Client};

use teloxide::{
    dispatching::Dispatcher,
    dptree,
    error_handlers::LoggingErrorHandler,
    update_listeners::{webhooks, UpdateListener},
    Bot,
};

use crate::{
    bot::bot_handler,
    routes::app_router,
    settings::{environment::Environment, stickers::Stickers, Settings},
};

pub async fn start_server(
    bot: Bot,
    settings: &Settings,
    env: &Environment,
) -> impl UpdateListener<Err = std::convert::Infallible> {
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    )
    .parse()
    .with_context(|| {
        format!(
            "{}:{}",
            settings.application.host, settings.application.port,
        )
    })
    .expect("unable to parse into address url");

    let url = format!("{}/webhook", settings.application.public_url)
        .parse()
        .context(settings.application.public_url.to_string())
        .expect("unable to parse into webhook url");

    let mut options = webhooks::Options::new(address, url);

    if *env == Environment::Local {
        options = options.drop_pending_updates();
    }

    let (mut listener, stop_flag, router) = webhooks::axum_to_router(bot, options)
        .await
        .expect("unable to get listener");

    let app = app_router(router);

    let stop_token = listener.stop_token();

    tokio::spawn(async move {
        axum::Server::bind(&address)
            .serve(app.into_make_service())
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|_| stop_token.stop())
            .expect("axum server error");
    });

    listener
}

pub async fn start_app(settings: Settings, env: &Environment) {
    let tele_bot = Bot::from_env();
    let stickers = Stickers::new().expect("error deserializing yaml for stickers");
    let chatgpt = Client::new();
    let listener = start_server(tele_bot.clone(), &settings, env).await;
    start_bot(tele_bot, listener, stickers, chatgpt).await;
}

pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    stickers: Stickers,
    chatgpt: Client<OpenAIConfig>,
) {
    let handler = bot_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![stickers, chatgpt])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
