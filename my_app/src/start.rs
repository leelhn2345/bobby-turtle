use std::time::Duration;

use async_openai::Client;
use my_app::{AppSettings, DatabaseSettings, Environment, Settings};
use sqlx::{postgres::PgPoolOptions, PgPool};
use teloxide::{
    update_listeners::webhooks::{self, Options},
    Bot,
};

use crate::jobs::init_scheduler;

fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

pub async fn start_app(settings: Settings, env: Environment) {
    let tele_bot = Bot::from_env();
    let chatgpt = Client::new();
    let connection_pool = get_connection_pool(&settings.database);

    let sched = init_scheduler(&tele_bot, &settings.stickers, &connection_pool);

    let options = get_webhook_options(&settings.application, &env);

    let Options { address, .. } = options;

    let (mut listener, stop_flag, router) = webhooks::axum_to_router(tele_bot.clone(), options)
        .await
        .map_err(|e| tracing::error!("{e:#?}"))
        .expect("unable to get listener");
}

fn get_webhook_options(settings: &AppSettings, env: &Environment) -> Options {
    let address = format!("{}:{}", settings.host, settings.port)
        .parse()
        .expect(
            format!(
                "unable to parse into address url - {}:{}",
                settings.host, settings.port
            )
            .as_ref(),
        );

    let url = format!("{}/webhook", settings.public_url)
        .parse()
        .expect(format!("unable to parse into webhook url - {}", settings.public_url).as_ref());

    let mut options = webhooks::Options::new(address, url);

    if *env == Environment::Local {
        options = options.drop_pending_updates();
        tracing::info!("app started in http://localhost:{}", settings.port);
    }

    options
}
