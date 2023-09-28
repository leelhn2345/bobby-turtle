use crate::handlers::owner::{OwnerGroupCommand, OwnerPrivateCommand};
use crate::handlers::system::*;
use crate::handlers::user::{UserGroupCommand, UserPrivateCommand};
use crate::handlers::vulgar::check_vulgar;
use crate::handlers::vulgar::scold_vulgar_message;
use crate::jobs;
use crate::settings::Environment;
use crate::settings::Settings;
use std::collections::HashSet;
use std::convert::Infallible;
use teloxide::dispatching::{Dispatcher, HandlerExt, MessageFilterExt, UpdateFilterExt};
use teloxide::dptree;

use teloxide::prelude::LoggingErrorHandler;
use teloxide::requests::Requester;

use teloxide::types::{Message, Update, UserId};
use teloxide::{update_listeners::UpdateListener, Bot};

use jobs::start_jobs;

use once_cell::sync::Lazy;
use std::sync::OnceLock;
use teloxide::types::Me;
use tracing::instrument;

static OWNERS: Lazy<HashSet<u64>> = Lazy::new(|| HashSet::from([2050440697, 220272763]));
static BOT_DETAILS: OnceLock<Me> = OnceLock::new();
pub static BOT_ME: Lazy<&Me> = Lazy::new(|| BOT_DETAILS.get().unwrap());

#[instrument(name = "set up bot details", skip_all)]
pub async fn setup_me(bot: &Bot) {
    let me = bot.get_me().await.expect("cannot get details about bot");
    let username = me.username.as_ref().unwrap();
    tracing::debug!("starting @{}", username);
    BOT_DETAILS.set(me).unwrap();
    tracing::info!("success");
}

pub fn check_is_owner(msg: Message) -> bool {
    msg.from()
        .map(|user| {
            let UserId(id) = user.id;
            OWNERS.contains(&id)
        })
        .unwrap_or_default()
}

/// Dispatch logic for teloxide bot
pub async fn start_bot(
    bot: Bot,
    listener: impl UpdateListener<Err = Infallible>,
    settings: Settings,
    env: Environment,
) {
    setup_me(&bot).await;

    start_jobs(&bot, &settings, env)
        .await
        .expect("cannot start job");

    let handler = dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| msg.chat.is_chat())
                        .branch(
                            dptree::filter(check_is_owner)
                                .filter_command::<OwnerGroupCommand>()
                                .endpoint(OwnerGroupCommand::parse_commands),
                        )
                        .branch(
                            dptree::entry()
                                .filter_command::<UserGroupCommand>()
                                .endpoint(UserGroupCommand::parse_commands),
                        ),
                )
                .branch(
                    dptree::filter(|msg: Message| msg.chat.is_private())
                        .branch(
                            dptree::filter(check_is_owner)
                                .filter_command::<OwnerPrivateCommand>()
                                .endpoint(OwnerPrivateCommand::parse_commands),
                        )
                        .branch(
                            dptree::entry()
                                .filter_command::<UserPrivateCommand>()
                                .endpoint(UserPrivateCommand::parse_commands),
                        ),
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
