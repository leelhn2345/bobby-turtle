use std::{convert::Infallible, future::Future, net::SocketAddr, time::Duration};

use anyhow::Context;
use async_openai::{config::OpenAIConfig, Client};

use axum::Router;
use sqlx::{postgres::PgPoolOptions, PgPool};
use teloxide::{
    dispatching::{dialogue::InMemStorage, Dispatcher},
    dptree,
    error_handlers::LoggingErrorHandler,
    stop::StopToken,
    update_listeners::{
        webhooks::{self, Options},
        UpdateListener,
    },
    Bot,
};
use tokio::signal;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    bot::{bot_handler, init_bot_details, CallbackState, ChatState},
    jobs::init_scheduler,
    routes::app_router,
    settings::{app::AppSettings, database::DatabaseSettings, environment::Environment, Settings},
};

async fn start_server(
    stop_token: StopToken,
    stop_flag: impl Future<Output = ()> + Send,
    router: Router,
    address: SocketAddr,
) {
    let app = app_router(router);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(stop_flag)
        .await
        .map_err(|_| stop_token.stop())
        .expect("axum server error");
}

fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

fn get_webhook_options(settings: &AppSettings, env: &Environment) -> Options {
    let address = format!("{}:{}", settings.host, settings.port)
        .parse()
        .context(format!("{}:{}", settings.host, settings.port,))
        .expect("unable to parse into address url");

    let url = format!("{}/webhook", settings.public_url)
        .parse()
        .context(settings.public_url.to_string())
        .expect("unable to parse into webhook url");

    let mut options = webhooks::Options::new(address, url);

    if *env == Environment::Local {
        options = options.drop_pending_updates();
        tracing::info!("app started in http://localhost:{}", settings.port);
    }

    options
}

pub async fn start_app(settings: Settings, env: Environment) {
    let tele_bot = Bot::from_env();
    let chatgpt = Client::new();
    let connection_pool = get_connection_pool(&settings.database);

    let sched = init_scheduler(&tele_bot, &settings.stickers, &connection_pool)
        .await
        .expect("cannot initialize scheduler");

    let options = get_webhook_options(&settings.application, &env);

    let Options { address, .. } = options;

    let (mut listener, stop_flag, router) = webhooks::axum_to_router(tele_bot.clone(), options)
        .await
        .map_err(|e| tracing::error!("{e:#?}"))
        .expect("unable to get listener");

    let stop_token = listener.stop_token();

    let axum_server = tokio::spawn(start_server(stop_token, stop_flag, router, address));

    let bot_app = start_bot(
        tele_bot,
        listener,
        settings,
        chatgpt,
        connection_pool,
        sched,
    );

    tokio::select! {
        _ = signal::ctrl_c() => tracing::info!("ctrl-c received"),
        _o = axum_server => tracing::info!("web server has shutdown."),
        _o = bot_app => tracing::info!("telegram bot app has shutdown."),
    }
}

async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
    chatgpt: Client<OpenAIConfig>,
    pool: PgPool,
    sched: JobScheduler,
) {
    init_bot_details(&bot).await;

    let handler = bot_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            settings.stickers,
            chatgpt,
            pool,
            InMemStorage::<ChatState>::new(),
            InMemStorage::<CallbackState>::new(),
            sched
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
