use crate::{
    settings::Settings,
    stickers::{send_many_stickers, send_sticker},
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
#[command(rename_rule = "lowercase", description = "hello friend!~ 🐢😊.")]
pub enum UserGroupCommand {
    Help,
    #[command(description = "a lovely hug! 🤗❤️")]
    Hug,
    // #[command(description = "greetings")]
    // Greet,
    #[command(description = "a passionate kiss")]
    Kiss,
    #[command(description = "give you my LOVE")]
    Love,
    #[command(description = "feed me!")]
    Feed,
    #[command(description = "IT'S PARTY TIME!! 🥳🥳")]
    Party,
    #[command(description = "the current date & time")]
    Now,
}
impl UserGroupCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: UserGroupCommand,
    ) -> MyResult<()> {
        match cmd {
            UserGroupCommand::Help => {
                bot.send_message(msg.chat.id, UserGroupCommand::descriptions().to_string())
                    .await?;
            }
            UserGroupCommand::Hug => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.hug).await?
            }
            UserGroupCommand::Kiss => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.kiss).await?
            }
            UserGroupCommand::Party => {
                send_many_stickers(&bot, &msg.chat.id, settings.stickers.party_animals).await?
            }
            UserGroupCommand::Love => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.love).await?
            }

            UserGroupCommand::Feed => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }
            UserGroupCommand::Now => {
                bot.send_message(msg.chat.id, datetime_now()).await?;
            }
        }
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(description = "Hello hello~~ 🐢🐢", rename_rule = "lowercase")]
pub enum UserPrivateCommand {
    Start,
    Help,
    #[command(parse_with = message_to_send, description="<ChatID> <Message>\n\t(Send message anonymously 😊)\n")]
    SendMessage(i64, String),
    #[command(description = "the current date & time")]
    Now,
}

impl UserPrivateCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: UserPrivateCommand,
    ) -> MyResult<()> {
        match cmd {
            UserPrivateCommand::Start => {
                let text = match msg.chat.username() {
                    Some(x) => format!("hello @{}! 🐢", x),
                    None => String::from("hello friend!"),
                };
                send_sticker(&bot, &msg.chat.id, settings.stickers.hello).await?;
                bot.send_message(msg.chat.id, text).await?;
            }
            UserPrivateCommand::Help => {
                bot.send_message(msg.chat.id, UserPrivateCommand::descriptions().to_string())
                    .await?;
            }
            UserPrivateCommand::SendMessage(chat_id, msg_string) => {
                bot.send_message(ChatId(chat_id), msg_string).await?;
            }
            UserPrivateCommand::Now => {
                bot.send_message(msg.chat.id, datetime_now()).await?;
            }
        }
        Ok(())
    }
}
