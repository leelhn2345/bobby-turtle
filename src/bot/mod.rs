mod command_handler;
mod message_handler;
mod stickers;

use std::convert::Infallible;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};

use self::command_handler::bot_command_handler;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::types::Update;
use teloxide::{dispatching::UpdateHandler, update_listeners::UpdateListener, Bot};

fn bot_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    let command_handler = bot_command_handler();
    use teloxide::dptree;

    dptree::entry().branch(Update::filter_message().branch(command_handler))
}
pub async fn start_bot(bot: Bot, listener: impl UpdateListener<Err = Infallible>) {
    // let handler = Update::filter_message()
    //     .enter_dialogue::<Message, InMemStorage<State>, State>()
    //     .branch(dptree::case![State::Start].endpoint(start))
    //     .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
    //     .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
    //     .branch(
    //         dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
    //     );
    let handler = bot_handler();

    Dispatcher::builder(bot, handler)
        // .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
