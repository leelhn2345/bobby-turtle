use chrono::{Datelike, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;
use teloxide::{
    payloads::SendMessageSetters, requests::Requester, types::Message, utils::command::BotCommands,
    Bot,
};

use crate::settings::stickers::Stickers;

use super::{calendar::calendar, chatroom::ChatRoom, send_sticker, BotDialogue, ChatState};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "See all available commands")]
    Help,
    #[command(description = "Chat with me!")]
    Chat,
    #[command(description = "Make me shutup")]
    Shutup,
    #[command(description = "Current month calendar")]
    Calendar,
    #[command(description = "Current datetime (GMT +8)")]
    DateTime,
    #[command(description = "Feed me")]
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
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        match cmd {
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
                bot.send_message(chat_id, "Yo yo yo, what do you wanna chat about?? 😊")
                    .await?;
            }
            Self::Shutup => {
                dialogue.update(ChatState::Shutup).await?;
                send_sticker(&bot, &chat_id, stickers.whatever).await?;
                bot.send_message(chat_id, "Huh?! Whatever 🙄. Byebye I'm off.")
                    .await?;
            }
            // Command::Chat(chat_msg) => bot_chat(bot, chatgpt, &msg, chat_msg, pool).await?,
            Self::Feed => {
                send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
                bot.send_message(chat_id, "~ feature coming soon ~").await?;
            }
            Self::Calendar => {
                let now = Utc::now().with_timezone(&Tz::Singapore);
                let calendar = calendar(now.day(), now.month(), now.year()).map_err(|e| {
                    tracing::error!("{e:#?}");
                    e
                })?;
                bot.send_message(chat_id, "🐢 Work in Progress 🐢")
                    .reply_markup(calendar)
                    .await?;
            }
        };
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum UserCommand {
    #[command()]
    Start,
}

impl UserCommand {
    pub async fn answer(
        bot: Bot,
        msg: Message,
        cmd: UserCommand,
        stickers: Stickers,
        pool: PgPool,
    ) -> anyhow::Result<()> {
        match cmd {
            UserCommand::Start => {
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
        }
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
