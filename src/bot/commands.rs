use async_openai::{config::OpenAIConfig, Client};
use chrono::Local;
use sqlx::PgPool;
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

use crate::{chat::bot_chat, settings::stickers::Stickers};

use super::{chatroom::ChatRoom, send_sticker};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command()]
    Help,
    #[command(description = "chat with me!")]
    Chat(String),
    #[command(description = "current datetime (GMT +8).")]
    DateTime,
    #[command(description = "feed me.")]
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
        chatgpt: Client<OpenAIConfig>,
        pool: PgPool,
    ) -> anyhow::Result<()> {
        let chat_id = msg.chat.id;
        match cmd {
            Command::Help => {
                bot.send_message(chat_id, Command::descriptions().to_string())
                    .await?
            }
            Command::DateTime => {
                let now = Local::now().format("%v\n%r").to_string();
                bot.send_message(chat_id, now).await?
            }
            Command::Chat(chat_msg) => bot_chat(bot, chatgpt, &msg, chat_msg, pool).await?,
            Command::Feed => {
                send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
                bot.send_message(chat_id, "~ feature coming soon ~").await?
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
