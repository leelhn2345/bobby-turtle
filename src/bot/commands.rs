use chrono::Utc;
use chrono_tz::Tz;
use sqlx::PgPool;
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

use crate::settings::stickers::Stickers;

use super::{
    callbacks::{occurence_page, CallbackState},
    chatroom::ChatRoom,
    sticker::send_sticker,
    BotDialogue, CallbackPage, ChatState,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "See all available commands")]
    Help,
    #[command(description = "Make bot respond to every message")]
    Chat,
    #[command(description = "Stop bot from responding to every message")]
    Shutup,
    #[command(description = "Let me remind you")]
    Remind,
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
        callback: CallbackState,
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
                bot.send_message(chat_id, "Hello! What do you wanna chat about?? 😊")
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
            Self::Remind => {
                callback.update(CallbackPage::Occcurence).await?;
                occurence_page(bot, msg.chat.id).await?;
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
