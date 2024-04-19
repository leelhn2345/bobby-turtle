mod commands;
mod member;
mod room;

use std::sync::OnceLock;

use anyhow::Result;
use teloxide::{
    dispatching::{DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt},
    dptree::{self, di::DependencyMap, Handler},
    requests::Requester,
    types::{ChatId, InputFile, Me, Message, Update},
    Bot,
};

use crate::chat::user_chat;

use self::member::{handle_me_left, i_got_added, i_got_removed};

/// feel free to `.unwrap()` once it has been initialized.
pub static BOT_ME: OnceLock<Me> = OnceLock::new();

pub async fn send_sticker(bot: &Bot, chat_id: &ChatId, sticker_id: String) -> anyhow::Result<()> {
    bot.send_sticker(*chat_id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn is_group_chat(msg: Message) -> bool {
    if msg.chat.is_private() || msg.chat.is_channel() {
        return false;
    }
    true
}

fn is_not_group_chat(msg: Message) -> bool {
    !is_group_chat(msg)
}

pub fn bot_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<commands::Command>()
                        .endpoint(commands::Command::answer),
                )
                .branch(
                    dptree::filter(is_not_group_chat)
                        .filter_command::<commands::UserCommand>()
                        .endpoint(commands::UserCommand::answer),
                )
                .branch(
                    Message::filter_new_chat_members()
                        .branch(dptree::filter(i_got_added).endpoint(member::handle_me_join))
                        .branch(dptree::endpoint(member::handle_member_join)),
                )
                .branch(
                    Message::filter_left_chat_member()
                        .branch(dptree::filter(i_got_removed).endpoint(handle_me_left))
                        .branch(dptree::endpoint(member::handle_member_leave)),
                )
                .branch(dptree::filter(is_not_group_chat).endpoint(user_chat)),
        )
}
