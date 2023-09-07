mod command;
mod message;

use std::convert::Infallible;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};

use crate::settings::Settings;
use crate::types::DpHandler;

use self::command::bot_command_handler;

use self::message::bot_message_handler;
use teloxide::dptree;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::types::Update;
use teloxide::{update_listeners::UpdateListener, Bot};

fn bot_handler() -> DpHandler {
    let command_handler = bot_command_handler();
    let message_handler = bot_message_handler();

    dptree::entry()
        // .inspect(|u: Update| {
        //     println!("{u:#?}"); // Print the update to the console with inspect
        //                         // method and a closure for debug purposes
        // })
        .branch(
            Update::filter_message()
                .branch(command_handler)
                .branch(message_handler),
        )
}
pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
) {
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
        // .dependencies(dptree::deps![settings])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
