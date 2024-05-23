use std::sync::OnceLock;

use anyhow::Result;
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt,
    },
    dptree::{self, di::DependencyMap, Handler},
    requests::Requester,
    types::{CallbackQuery, Me, Message, Update},
    utils::command::BotCommands,
    Bot,
};

use crate::{
    callbacks::{
        change_time_callback, confirm_reminder_text, date_callback, expired_callback,
        occurence_callback, remind_text_callback, time_callback, CallbackPage,
    },
    chat::user_chat,
    chatroom::ChatRoom,
    commands,
    handlers::{group_title_change, is_not_group_chat},
    member::{self, handle_me_leave, i_got_added, i_got_removed},
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

pub type BotDialogue = Dialogue<ChatState, InMemStorage<ChatState>>;

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
}

pub fn bot_handler() -> Handler<'static, DependencyMap, Result<()>, DpHandlerDescription> {
    dptree::entry()
        .inspect(|u: Update| tracing::debug!("{:#?}", u))
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<ChatState>, ChatState>()
                .enter_dialogue::<Message, InMemStorage<CallbackPage>, CallbackPage>()
                .branch(
                    dptree::case![CallbackPage::ConfirmDateTime { date_time }]
                        .endpoint(confirm_reminder_text),
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
                .enter_dialogue::<CallbackQuery, InMemStorage<CallbackPage>, CallbackPage>()
                .branch(dptree::case![CallbackPage::Occcurence].endpoint(occurence_callback))
                .branch(dptree::case![CallbackPage::RemindDate].endpoint(date_callback))
                .branch(
                    dptree::case![CallbackPage::RemindDateTime { date, time }]
                        .endpoint(time_callback),
                )
                .branch(
                    dptree::case![CallbackPage::ConfirmDateTime { date_time }]
                        .endpoint(change_time_callback),
                )
                .branch(
                    dptree::case![CallbackPage::ConfirmOneOffJob {
                        date_time,
                        msg_text
                    }]
                    .endpoint(remind_text_callback),
                )
                .branch(dptree::endpoint(expired_callback)),
        )
}
