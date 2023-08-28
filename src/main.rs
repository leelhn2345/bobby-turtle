use axum::{routing, Router};
use polarrbear_bot::routes::health_check::health_check;
use polarrbear_bot::settings::get_settings;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::update_listeners::{webhooks, UpdateListener};
use teloxide::{dptree, Bot};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(polarrbear_bot::routes::health_check::health_check),
        tags((name = "polarbearr", description = "sample axum swagger integration"))
    )]
    struct ApiDoc;

    let settings = get_settings().expect("failed to read settings");
    let bot = Bot::from_env();

    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );

    let url = settings.application.base_url;

    let options = webhooks::Options::new(address.parse().unwrap(), url.parse().unwrap());

    let (mut update_listener, stop_flag, bot_router) =
        webhooks::axum_to_router(bot.clone(), options)
            .await
            .unwrap();

    let stop_token = update_listener.stop_token();
    let app = bot_router
        .merge(SwaggerUi::new("/docs").url("/docs.json", ApiDoc::openapi()))
        .route("/", routing::get(|| async { "Hello, World!" }))
        .route("/health_check", routing::get(health_check));

    // run it with hyper on localhost:3000
    tokio::spawn(async move {
        axum::Server::bind(&address.parse().unwrap())
            .serve(app.into_make_service())
            .with_graceful_shutdown(stop_flag)
            .await
            .map_err(|err| {
                stop_token.stop();
                err
            })
            .expect("axum server error");
    });

    // let handler = dptree::entry()
    //     .branch(smiles_handler())
    //     .branch(sqrt_handler())
    //     .branch(not_found_handler());

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
            .branch(
                dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(update_listener, LoggingErrorHandler::new())
    .await;
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    ReceiveAge {
        full_name: String,
    },
    ReceiveLocation {
        full_name: String,
        age: u8,
    },
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?")
        .await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn receive_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "How old are you?").await?;
            dialogue
                .update(State::ReceiveAge {
                    full_name: text.into(),
                })
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    full_name: String, // Available from `State::ReceiveAge`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            bot.send_message(msg.chat.id, "What's your location?")
                .await?;
            dialogue
                .update(State::ReceiveLocation { full_name, age })
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number.").await?;
        }
    }

    Ok(())
}

async fn receive_location(
    bot: Bot,
    dialogue: MyDialogue,
    (full_name, age): (String, u8), // Available from `State::ReceiveLocation`.
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(location) => {
            let report = format!("Full name: {full_name}\nAge: {age}\nLocation: {location}");
            bot.send_message(msg.chat.id, report).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}
