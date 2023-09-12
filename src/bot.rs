use crate::handlers::command::*;
use crate::handlers::system::*;
use crate::settings::Settings;
use crate::types::MyResult;
use std::collections::HashSet;
use std::convert::Infallible;
use teloxide::dispatching::HandlerExt;
use teloxide::dispatching::MessageFilterExt;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::requests::Requester;
use teloxide::types::UserId;
use teloxide::types::{Message, Update};
use teloxide::{update_listeners::UpdateListener, Bot};

fn check_is_owner(msg: Message, owners: &HashSet<u64>) -> bool {
    let user = msg.from();
    let Some(user_id) = user else { return false };
    let UserId(id) = user_id.id;
    owners.contains(&id)
}

async fn dummy_func(bot: Bot, msg: Message) -> MyResult<()> {
    bot.send_message(msg.chat.id, "hello").await?;
    Ok(())
}

pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
) {
    let owners: HashSet<u64> = HashSet::from([2050440697, 220272763]);

    let handler = dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| msg.chat.is_private())
                        .filter_command::<PrivateCommand>()
                        .endpoint(PrivateCommand::parse_commands),
                )
                .branch(
                    dptree::filter(move |msg: Message| check_is_owner(msg, &owners))
                        .filter_command::<OwnerCommand>()
                        .endpoint(OwnerCommand::parse_commands),
                )
                .branch(
                    teloxide::filter_command::<UserCommand, _>()
                        // .filter(|msg: Message| msg.chat.is_private())
                        .endpoint(UserCommand::parse_commands),
                )
                .branch(Message::filter_new_chat_members().endpoint(handle_new_member))
                .branch(Message::filter_left_chat_member().endpoint(handle_left_member)),
        )
        .branch(Update::filter_edited_message().endpoint(dummy_func));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![settings])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
