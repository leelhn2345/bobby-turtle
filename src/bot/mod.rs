mod command_handler;
mod message_handler;
mod stickers;

use std::convert::Infallible;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};

use teloxide::prelude::LoggingErrorHandler;
use teloxide::requests::Requester;
use teloxide::types::{Message, Update};
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::UpdateHandler, update_listeners::UpdateListener, Bot};

use self::command_handler::bot_command_handler;

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
    // use teloxide::dptree;
    // let handler = Update::filter_message()
    //     // You can use branching to define multiple ways in which an update will be handled. If the
    //     // first branch fails, an update will be passed to the second branch, and so on.
    //     .branch(
    //         dptree::entry()
    //             // Filter commands: the next handlers will receive a parsed `SimpleCommand`.
    //             .filter_command::<SimpleCommand>()
    //             // If a command parsing fails, this handler will not be executed.
    //             .endpoint(simple_commands_handler),
    //     );

    Dispatcher::builder(bot, handler)
        // .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Simple commands")]
enum SimpleCommand {
    #[command(description = "shows this message.")]
    Help,
    #[command(description = "shows maintainer info.")]
    Maintainer,
    #[command(description = "shows your ID.")]
    MyId,
}

async fn simple_commands_handler(
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: SimpleCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        SimpleCommand::Help => {
            if msg.chat.is_group() || msg.chat.is_supergroup() {
                SimpleCommand::descriptions()
                    .username_from_me(&me)
                    .to_string()
            } else {
                SimpleCommand::descriptions().to_string()
            }
        }

        SimpleCommand::MyId => {
            format!("{}", msg.from().unwrap().id)
        }
        SimpleCommand::Maintainer => todo!(),
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
