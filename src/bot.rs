use crate::handlers::command::*;
use crate::handlers::system::*;
use crate::handlers::vulgar::check_vulgar;
use crate::handlers::vulgar::scold_vulgar_message;
use crate::settings::Settings;
use crate::types::MyResult;
use std::collections::HashSet;
use std::convert::Infallible;

use once_cell::sync::Lazy;
use std::sync::OnceLock;
use teloxide::dispatching::MessageFilterExt;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::requests::Requester;
use teloxide::types::UserId;
use teloxide::types::{Me, Message, Update};
use teloxide::{update_listeners::UpdateListener, Bot};

pub static BOT_ME: Lazy<&Me> = Lazy::new(|| BOT_DETAILS.get().unwrap());

static BOT_DETAILS: OnceLock<Me> = OnceLock::new();

async fn setup_me(bot: &Bot) {
    let me = bot.get_me().await.expect("cannot get details about bot");

    BOT_DETAILS.set(me).unwrap();
}

#[tracing::instrument(skip_all)]
fn check_is_owner(msg: Message, owners: &HashSet<u64>) -> bool {
    let user = msg.from();
    let Some(user_id) = user else { return false };
    let UserId(id) = user_id.id;
    tracing::info!("yes it is owner!");
    owners.contains(&id)
}

#[tracing::instrument(skip_all)]
async fn dummy_func(bot: Bot, msg: Message) -> MyResult<()> {
    bot.send_message(msg.chat.id, "hello").await?;
    Ok(())
}

pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
) {
    setup_me(&bot).await;
    let owners: HashSet<u64> = HashSet::from([2050440697, 220272763]);

    let handler = dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    teloxide::filter_command::<PrivateCommand, _>()
                        .filter(|msg: Message| msg.chat.is_private())
                        .endpoint(PrivateCommand::parse_commands),
                )
                .branch(
                    teloxide::filter_command::<OwnerCommand, _>()
                        .filter(move |msg: Message| check_is_owner(msg, &owners))
                        .endpoint(OwnerCommand::parse_commands),
                )
                .branch(
                    teloxide::filter_command::<UserCommand, _>()
                        // .filter(|msg: Message| msg.chat.is_private())
                        .endpoint(UserCommand::parse_commands),
                )
                .branch(Message::filter_new_chat_members().endpoint(handle_new_member))
                .branch(Message::filter_left_chat_member().endpoint(handle_left_member))
                .branch(dptree::filter(check_vulgar).endpoint(scold_vulgar_message)),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![settings])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}
