mod calendar;
mod chatroom;
mod commands;
mod handlers;
mod job_text;
mod member;
mod occurence;
mod time_pick;

use std::sync::OnceLock;

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt,
    },
    dptree::{self, di::DependencyMap, Handler},
    requests::Requester,
    types::{CallbackQuery, Chat, ChatId, InputFile, Me, Message, MessageId, Update},
    utils::command::BotCommands,
    Bot,
};

use crate::chat::user_chat;

use self::{
    calendar::calendar_callback,
    chatroom::ChatRoom,
    handlers::{group_title_change, is_not_group_chat},
    job_text::{one_off_job_callback, register_job_text},
    member::{handle_me_leave, i_got_added, i_got_removed},
    occurence::occurence_callback,
    time_pick::{change_time_callback, time_pick_callback, RemindTime},
};

/// feel free to `.unwrap()` once it has been initialized.
pub static BOT_ME: OnceLock<Me> = OnceLock::new();
/// feel free to `.unwrap()` once it has been initialized.
pub static BOT_NAME: OnceLock<String> = OnceLock::new();

#[derive(Clone, Default)]
pub enum ChatState {
    #[default]
    Shutup,
    Talk,
}

#[derive(Clone, Default)]
pub enum CallbackState {
    #[default]
    Expired,
    Occcurence,
    RemindDate,
    RemindDateTime {
        date: NaiveDate,
        time: RemindTime,
    },
    ConfirmDateTime {
        date_time: DateTime<Tz>,
    },
    ConfirmOneOffJob {
        date_time: DateTime<Tz>,
        msg_text: String,
    },
}

pub type BotDialogue = Dialogue<ChatState, InMemStorage<ChatState>>;
pub type CallbackDialogue = Dialogue<CallbackState, InMemStorage<CallbackState>>;

pub async fn init_bot_details(bot: &Bot) {
    bot.set_my_commands(commands::Command::bot_commands())
        .await
        .expect("error setting bot commands.");

    let me = bot.get_me().await.expect("cannot get details about bot.");
    let first_name_vec: Vec<&str> = me.first_name.split_whitespace().collect();
    let name = first_name_vec.first().unwrap().to_lowercase();

    BOT_NAME
        .set(name)
        .expect("cannot set bot's name as static value.");
    BOT_ME
        .set(me)
        .expect("error setting bot details to static value.");
    tracing::debug!("{BOT_ME:#?}");
}

pub async fn send_sticker(bot: &Bot, chat_id: &ChatId, sticker_id: String) -> anyhow::Result<()> {
    bot.send_sticker(*chat_id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn expired_callback_endpt(bot: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    let Some(Message { id, chat, .. }) = q.message else {
        tracing::error!("no message data from telegram");
        bail!("no query message")
    };
    expired_callback_msg(bot, chat, id).await?;
    Ok(())
}

pub async fn expired_callback_msg(bot: Bot, chat: Chat, id: MessageId) -> anyhow::Result<()> {
    bot.edit_message_text(chat.id, id, "This has expired 😅 🐢🐢🐢")
        .await?;
    Ok(())
}

pub fn bot_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<ChatState>, ChatState>()
                .enter_dialogue::<Message, InMemStorage<CallbackState>, CallbackState>()
                .branch(
                    dptree::case![CallbackState::ConfirmDateTime { date_time }]
                        .endpoint(register_job_text),
                )
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
                .branch(dptree::case![ChatState::Talk].endpoint(user_chat)), // .branch(dptree::filter(to_bot).endpoint(user_chat)),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<CallbackState>, CallbackState>()
                .branch(dptree::case![CallbackState::Occcurence].endpoint(occurence_callback))
                .branch(dptree::case![CallbackState::RemindDate].endpoint(calendar_callback))
                .branch(
                    dptree::case![CallbackState::RemindDateTime { date, time }]
                        .endpoint(time_pick_callback),
                )
                .branch(
                    dptree::case![CallbackState::ConfirmDateTime { date_time }]
                        .endpoint(change_time_callback),
                )
                .branch(
                    dptree::case![CallbackState::ConfirmOneOffJob {
                        date_time,
                        msg_text
                    }]
                    .endpoint(one_off_job_callback),
                )
                .branch(dptree::case![CallbackState::Expired].endpoint(expired_callback_endpt))
                .branch(dptree::endpoint(expired_callback_endpt)),
        )
}
