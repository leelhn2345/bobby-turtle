use teloxide::{
    update_listeners::{webhooks, UpdateListener},
    Bot,
};

use crate::{routes::setup_bot_router, settings::Settings};

pub async fn setup_axum_webhook(
    settings: &Settings,
    bot: Bot,
) -> impl UpdateListener<Err = std::convert::Infallible> {
    // let options = webhooks::Options::new(address, url);
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    )
    .parse()
    .expect("unable to parse host and/or port");

    let url = format!("{}/webhook", settings.application.base_url)
        .parse()
        .expect("unable to parse base url");

    let options = webhooks::Options::new(address, url);

    let (mut listener, stop_flag, bot_router) = webhooks::axum_to_router(bot, options)
        .await
        .expect("unable to get listener");

    let router = setup_bot_router(bot_router);

    let stop_token = listener.stop_token();

    tokio::spawn(async move {
        axum::Server::bind(&address)
            .serve(router.into_make_service())
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|_| stop_token.stop())
            .expect("axum server error")
    });

    listener
}
