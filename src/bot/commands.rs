use async_openai::{config::OpenAIConfig, Client};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, InputFile, Message, ParseMode},
    utils::command::BotCommands,
    Bot,
};

use crate::{
    chat::{chatgpt_chat, ChatError},
    settings::stickers::Stickers,
};

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
    #[command(description = "current datetime.")]
    DateTime,
    #[command(description = "feed me.")]
    Feed,
}

#[tracing::instrument(name = "answer commands", skip_all)]
#[allow(deprecated)]
pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    stickers: Stickers,
    chatgpt: Client<OpenAIConfig>,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;
    match cmd {
        Command::Help => {
            bot.send_message(chat_id, Command::descriptions().to_string())
                .await?
        }
        Command::DateTime => {
            send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
            bot.send_message(chat_id, "~ feature coming soon ~").await?
        }
        Command::Chat(chat_msg) => match chatgpt_chat(chatgpt, &msg, chat_msg).await {
            Ok(response) => {
                bot.send_message(chat_id, response)
                    .parse_mode(ParseMode::Markdown)
                    .reply_to_message_id(msg.id)
                    .await?
            }
            Err(e) => {
                if let ChatError::EmptyMessageFromUser(empty_msg_response) = e {
                    bot.send_message(chat_id, empty_msg_response)
                        .parse_mode(ParseMode::Markdown)
                        .reply_to_message_id(msg.id)
                        .await?
                } else {
                    tracing::error!("{:#?}", e);
                    return Err(e.into());
                }
            }
        },
        Command::Feed => {
            send_sticker(&bot, &chat_id, stickers.coming_soon).await?;
            bot.send_message(chat_id, "~ feature coming soon ~").await?
        }
    };
    Ok(())
}

pub async fn send_sticker(bot: &Bot, chat_id: &ChatId, sticker_id: String) -> anyhow::Result<()> {
    bot.send_sticker(*chat_id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}
