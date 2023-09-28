use crate::handlers::owner::OwnerCommand;
use crate::handlers::private_chat::PrivateCommand;
use crate::handlers::system::*;
use crate::handlers::user::UserCommand;
use crate::handlers::vulgar::check_vulgar;
use crate::handlers::vulgar::scold_vulgar_message;
use crate::jobs;
use crate::settings::Environment;
use crate::settings::Settings;
use crate::types::MyResult;
use std::collections::HashSet;
use std::convert::Infallible;

use teloxide::dispatching::MessageFilterExt;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::LoggingErrorHandler;
use teloxide::requests::Requester;

use teloxide::types::UserId;
use teloxide::types::{Message, Update};
use teloxide::{update_listeners::UpdateListener, Bot};

use jobs::start_jobs;

use once_cell::sync::Lazy;
use std::sync::OnceLock;
use teloxide::types::Me;
use tracing::instrument;

pub static BOT_ME: Lazy<&Me> = Lazy::new(|| BOT_DETAILS.get().unwrap());

static BOT_DETAILS: OnceLock<Me> = OnceLock::new();

#[instrument(name = "set up bot details", skip_all)]
pub async fn setup_me(bot: &Bot) {
    let me = bot.get_me().await.expect("cannot get details about bot");
    let username = me.username.as_ref().unwrap();
    tracing::debug!("starting {}", username);
    BOT_DETAILS.set(me).unwrap();
    tracing::info!("success");
}

#[tracing::instrument(skip_all)]
fn check_is_owner(msg: Message, owners: &HashSet<u64>) -> bool {
    msg.from()
        .map(|user| {
            let UserId(id) = user.id;
            owners.contains(&id)
        })
        .unwrap_or_default()
}

#[tracing::instrument(skip_all)]
async fn dummy_func(bot: Bot, msg: Message) -> MyResult<()> {
    bot.send_message(msg.chat.id, "hello").await?;
    Ok(())
}

/// Dispatch logic for teloxide bot
pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
    env: Environment,
) {
    setup_me(&bot).await;
    let owners: HashSet<u64> = HashSet::from([2050440697, 220272763]);

    start_jobs(&bot, &settings, env)
        .await
        .expect("cannot start job");
    // bot.send_message(ChatId(-1001838253386), "hello")
    //     .await
    //     .expect("this line has runtime error");

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
                        .endpoint(OwnerCommand::parse_group_commands),
                )
                .branch(
                    teloxide::filter_command::<UserCommand, _>()
                        // .filter(|msg: Message| msg.chat.is_private())
                        .endpoint(UserCommand::parse_group_commands),
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
