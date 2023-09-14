use crate::{
    settings::Settings,
    stickers::send_sticker,
    types::MyResult,
    utils::{datetime_now, message_to_send},
};
use teloxide::{
    requests::Requester,
    types::{ChatId, Message},
    utils::command::BotCommands,
    Bot,
};

#[derive(BotCommands, Clone)]
#[command(description = "Hello hello~~ 🐢🐢", rename_rule = "lowercase")]
pub enum PrivateCommand {
    Start,
    Help,
    #[command(parse_with = message_to_send,description="send message anonymously 😊")]
    SendMessage(i64, String),
    #[command(description = "the current date & time")]
    Now,
}

impl PrivateCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: PrivateCommand,
    ) -> MyResult<()> {
        match cmd {
            PrivateCommand::Start => {
                let text = match msg.chat.username() {
                    Some(x) => format!("hello @{}! 🐢", x),
                    None => String::from("hello friend!"),
                };
                send_sticker(&bot, &msg.chat.id, settings.stickers.hello).await?;
                bot.send_message(msg.chat.id, text).await?;
            }
            PrivateCommand::Help => {
                bot.send_message(msg.chat.id, PrivateCommand::descriptions().to_string())
                    .await?;
            }
            PrivateCommand::SendMessage(chat_id, msg_string) => {
                bot.send_message(ChatId(chat_id), msg_string).await?;
            }
            PrivateCommand::Now => {
                bot.send_message(msg.chat.id, datetime_now()).await?;
            }
        }
        Ok(())
    }
}
