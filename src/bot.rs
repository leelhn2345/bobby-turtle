use crate::handlers::admin::AdminCommand;
use crate::handlers::system::*;
use crate::handlers::user::{UserGroupCommand, UserPrivateCommand};
use crate::handlers::vulgar::check_vulgar;
use crate::handlers::vulgar::scold_vulgar_message;
use crate::jobs;
use crate::settings::Environment;
use crate::settings::Settings;
use crate::types::MyError;
use crate::web::setup_axum_webhook;
use std::collections::HashSet;

use teloxide::dispatching::{
    Dispatcher, DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt,
};
use teloxide::dptree;

use teloxide::prelude::{DependencyMap, Handler, LoggingErrorHandler};
use teloxide::requests::Requester;

use teloxide::types::{Message, Update, UserId};
use teloxide::Bot;

use jobs::start_jobs;

use once_cell::sync::Lazy;
use std::sync::OnceLock;
use teloxide::types::Me;
use tracing::instrument;

static OWNERS: Lazy<HashSet<u64>> = Lazy::new(|| HashSet::from([2050440697, 220272763]));
static BOT_DETAILS: OnceLock<Me> = OnceLock::new();
pub static BOT_ME: Lazy<&Me> = Lazy::new(|| BOT_DETAILS.get().unwrap());

#[instrument(name = "set up bot details", skip_all)]
async fn setup_me(bot: &Bot) {
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
pub async fn start_bot(settings: Settings, env: Environment) {
    let bot = Bot::from_env();
    let listener = setup_axum_webhook(&settings, bot.clone()).await;

    setup_me(&bot).await;

    start_jobs(&bot, &settings, env)
        .await
        .expect("cannot start job");

    let handler = setup_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![settings])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

type TeleHandler<'a> = Handler<'a, DependencyMap, Result<(), MyError>, DpHandlerDescription>;

fn setup_handler() -> TeleHandler<'static> {
    dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| msg.chat.is_supergroup() || msg.chat.is_group())
                        .filter_command::<UserGroupCommand>()
                        .endpoint(UserGroupCommand::parse_commands),
                )
                .branch(
                    dptree::filter(|msg: Message| msg.chat.is_private())
                        .branch(
                            dptree::filter(check_is_owner)
                                .filter_command::<AdminCommand>()
                                .endpoint(AdminCommand::parse_commands),
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
        )
}
