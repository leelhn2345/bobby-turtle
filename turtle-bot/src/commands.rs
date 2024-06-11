use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use chrono_tz::Tz;
use gaia::stickers::Stickers;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use sqlx::PgPool;
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

use crate::{
    bot::{BotDialogue, ChatState},
    callbacks::CallbackPage,
    chatroom::ChatRoom,
    handlers::{is_group_chat, is_not_group_chat},
};

use super::{
    callbacks::{new_occurence_page, CallbackState},
    sticker::send_sticker,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(hide)]
    Start,
    #[command(hide)]
    Register,
    #[command(hide)]
    Whisper,
    /// See all available commands
    Help,
    /// Make bot respond to messages
    Chat,
    /// Stop bot from responding to messages
    Shutup,
    /// Set reminder
    Remind,
    /// Current datetime (GMT+8)
    DateTime,
    #[command(hide)]
    Feed,
}
impl Command {
    #[tracing::instrument(name = "answer commands", skip_all)]
    #[allow(deprecated)]
    pub async fn answer(
        bot: Bot,
        msg: Message,
        cmd: Command,
        stickers: Stickers,
        dialogue: BotDialogue,
        callback: CallbackState,
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        let user = msg.from().ok_or_else(|| anyhow!("not a valid user"))?;
        let username = (user.username).as_ref();
        let user_id = user.id.0;
        let user_id_i64 = i64::from_le_bytes(user_id.to_le_bytes());
        match cmd {
            Self::Whisper => {
                if is_not_group_chat(msg.clone()) {
                    bot.send_message(chat_id, "this command can only be used in group chats")
                        .await?;
                } else {
                    let exists = sqlx::query_scalar!(
                        "select exists 
                        (select telegram_user_id from telegram_users 
                         where telegram_user_id = $1)
                        ",
                        user_id_i64
                    )
                    .fetch_one(&pool)
                    .await?
                    .context("missing row")?;
                    if exists {
                        sqlx::query!(
                            "insert into telegram_whisperers
                            (telegram_user_id, telegram_chat_id, registered)
                            values ($1, $2, $3)",
                            user_id_i64,
                            chat_id.0,
                            Utc::now()
                        )
                        .execute(&pool)
                        .await?;
                        let name = match username {
                            Some(x) => format!("@{x}"),
                            None => user.first_name.to_string(),
                        };
                        let text = format!("I am now a whisperer for {name}");
                        bot.send_message(chat_id, text).await?;
                    } else {
                        send_sticker(&bot, &chat_id, stickers.lame).await?;
                        bot.send_message(chat_id, "you're not a registered whisperer 🙄")
                            .await?;
                    }
                }
            }
            Self::Register => {
                if is_group_chat(msg.clone()) {
                    bot.send_message(chat_id, "this command can only be used in individual chats")
                        .await?;
                } else {
                    let exists = sqlx::query_scalar!(
                        "select exists 
                        (select telegram_user_id from telegram_users 
                         where telegram_user_id = $1)
                        ",
                        user_id_i64
                    )
                    .fetch_one(&pool)
                    .await?
                    .context("missing row")?;

                    if exists {
                        bot.send_message(chat_id, "you are already a whisperer")
                            .await?;
                    } else {
                        let token = Alphanumeric.sample_string(&mut thread_rng(), 16);
                        let expiry = Utc::now() + Duration::minutes(3);
                        sqlx::query!(
                            "insert into telegram_tokens
                            (telegram_token, telegram_user_id, telegram_username, expiry)
                            values ($1, $2, $3, $4)
                            on conflict (telegram_user_id) do update
                            set telegram_token = excluded.telegram_token, expiry = excluded.expiry
                            ",
                            token,
                            user_id_i64,
                            username,
                            expiry
                        )
                        .execute(&pool)
                        .await?;
                        bot.send_message(chat_id, "token is only valid for 3 minutes")
                            .await?;
                        bot.send_message(chat_id, token).await?;
                    }
                }
            }
            Self::Help => {
                bot.send_message(chat_id, Command::descriptions().to_string())
                    .await?;
            }
            Self::DateTime => {
                let now = Utc::now()
                    .with_timezone(&Tz::Singapore)
                    .format("%v\n%r")
                    .to_string();
                bot.send_message(chat_id, now).await?;
            }
            Self::Chat => {
                dialogue.update(ChatState::Talk).await?;
                send_sticker(&bot, &chat_id, stickers.hello).await?;
                bot.send_message(chat_id, "Hello! What do you wanna chat about?? 😊")
                    .await?;
            }
            Self::Shutup => {
                dialogue.update(ChatState::Shutup).await?;
                send_sticker(&bot, &chat_id, stickers.whatever).await?;
                bot.send_message(chat_id, "Huh?! Whatever 🙄. Byebye I'm off.")
                    .await?;
            }
            Self::Feed => {
                send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
                bot.send_message(chat_id, "~ feature coming soon ~").await?;
            }
            Self::Remind => {
                callback.update(CallbackPage::Occcurence).await?;
                new_occurence_page(bot, msg.chat.id).await?;
            }
            Self::Start => {
                let chat_room = ChatRoom::new(&msg);
                chat_room.save(&pool).await?;

                let username = msg.chat.username();
                let text = if let Some(name) = username {
                    format!("Hello @{name}! 🐢")
                } else {
                    "Hello friend! 🐢".to_string()
                };
                send_sticker(&bot, &msg.chat.id, stickers.hello).await?;
                bot.send_message(msg.chat.id, text).await?;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use chrono_tz::Tz;

    #[test]
    fn timezone() {
        let timezone = Tz::America__Chicago;

        // Get the current date and time in the specified location
        let wow = Utc::now().with_timezone(&timezone);
        // .format("%v\n%r")
        // .to_string();

        println!("{wow}");
    }
}
