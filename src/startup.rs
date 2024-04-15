use std::convert::Infallible;

use axum::{body::Body, http::Request, routing::get};
use teloxide::{
    dispatching::Dispatcher,
    dptree,
    error_handlers::LoggingErrorHandler,
    update_listeners::{webhooks, UpdateListener},
    Bot,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use utoipa::OpenApi;
use utoipa_auto_discovery::utoipa_auto_discovery;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    bot::bot_handler,
    routes::health_check,
    settings::{environment::Environment, utility::Utility, Settings},
};

#[utoipa_auto_discovery(paths = "
    ( health_check => ./src/routes/health_check.rs );
")]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

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
    .expect("unable to parse into address url");

    let url = format!("{}/webhook", settings.application.public_url)
        .parse()
        .expect("unable to parse into webhook url");

    let mut options = webhooks::Options::new(address, url);

    if *env == Environment::Local {
        options = options.drop_pending_updates();
    }

    let (mut listener, stop_flag, router) = webhooks::axum_to_router(bot, options)
        .await
        .expect("unable to get listener");

    let trace_layer = ServiceBuilder::new().layer(TraceLayer::new_for_http().make_span_with(
        |request: &Request<Body>| {
            let req_id = uuid::Uuid::new_v4();
            tracing::info_span!(
                "request",
                method = tracing::field::display(request.method()),
                uri = tracing::field::display(request.uri()),
                req_id = tracing::field::display(req_id)
            )
        },
    ));

    let app = router
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", get(health_check::root))
        .route("/health_check", get(health_check::health_check))
        .layer(trace_layer);

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

#[tracing::instrument(
    name = "starting app"
    skip_all
)]
pub async fn start_app(settings: Settings, env: &Environment) {
    let tele_bot = Bot::from_env();
    let listener = start_server(tele_bot.clone(), &settings, env).await;
    start_bot(tele_bot, listener).await;
}

pub async fn start_bot(bot: Bot, listener: impl UpdateListener<Err = Infallible>) {
    let util = Utility::new().expect("error deserializing yaml for utility");
    let handler = bot_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![util])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
