mod chatroom;
mod commands;
mod filters;
mod member;

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

use self::{
    chatroom::ChatRoom,
    filters::{group_title_change, is_not_group_chat, to_bot},
    member::{handle_me_leave, i_got_added, i_got_removed},
};

/// feel free to `.unwrap()` once it has been initialized.
pub static BOT_ME: OnceLock<Me> = OnceLock::new();
pub static BOT_NAME: OnceLock<String> = OnceLock::new();

pub async fn send_sticker(bot: &Bot, chat_id: &ChatId, sticker_id: String) -> anyhow::Result<()> {
    bot.send_sticker(*chat_id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
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
                        .branch(dptree::filter(i_got_removed).endpoint(handle_me_leave))
                        .branch(dptree::endpoint(member::handle_member_leave)),
                )
                .branch(dptree::filter(group_title_change).endpoint(ChatRoom::update_title))
                .branch(dptree::filter(is_not_group_chat).endpoint(user_chat))
                .branch(dptree::filter(to_bot).endpoint(user_chat)),
        )
}
