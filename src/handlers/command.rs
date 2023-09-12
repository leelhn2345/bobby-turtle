use crate::{
    settings::Settings,
    stickers::{send_many_stickers, send_sticker},
    types::MyResult,
};
use teloxide::{
    requests::Requester,
    types::{ChatId, Message},
    utils::command::{BotCommands, ParseError},
    Bot,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "hello owner!~ 🐢😊. \nthese are the available commands"
)]
pub enum OwnerCommand {
    #[command(description = "list down all commands")]
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
}
impl OwnerCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: OwnerCommand,
    ) -> MyResult<()> {
        match cmd {
            OwnerCommand::Help => {
                bot.send_message(msg.chat.id, OwnerCommand::descriptions().to_string())
                    .await?;
            }
            OwnerCommand::Hug => send_sticker(&bot, &msg, settings.stickers.hug).await?,
            OwnerCommand::Kiss => send_sticker(&bot, &msg, settings.stickers.kiss).await?,
            OwnerCommand::Party => {
                send_many_stickers(&bot, &msg, settings.stickers.party_animals).await?
            }
            OwnerCommand::Love => send_sticker(&bot, &msg, settings.stickers.love).await?,

            OwnerCommand::Feed => {
                send_sticker(&bot, &msg, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }
        }
        Ok(())
    }
}

fn message_to_send(input: String) -> Result<(i64, String), ParseError> {
    let mut parts = input.splitn(2, ' ');

    let chat_id = parts
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|e| ParseError::IncorrectFormat(e.into()))?;

    let message = parts.next().unwrap_or("yo yo yo").into();

    Ok((chat_id, message))
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "hello user 🐢😊~. \nThese are the available commands."
)]
pub enum UserCommand {
    #[command(description = "list down all commands")]
    Help,
    #[command(description = "IT'S PARTY TIME!! 🥳🥳")]
    Party,
    #[command(description = "feed me!")]
    Feed,
}

impl UserCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: UserCommand,
    ) -> MyResult<()> {
        match cmd {
            UserCommand::Help => {
                bot.send_message(msg.chat.id, UserCommand::descriptions().to_string())
                    .await?;
            }
            UserCommand::Party => {
                send_many_stickers(&bot, &msg, settings.stickers.party_animals).await?
            }

            UserCommand::Feed => {
                send_sticker(&bot, &msg, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(description = "Hello hello~~ 🐢🐢", rename_rule = "lowercase")]
pub enum PrivateCommand {
    Start,
    Help,
    #[command(parse_with = message_to_send,description="send message anonymously 😊")]
    SendMessage(i64, String),
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
                send_sticker(&bot, &msg, settings.stickers.hello).await?;
                bot.send_message(msg.chat.id, text).await?;
            }
            PrivateCommand::Help => {
                bot.send_message(msg.chat.id, PrivateCommand::descriptions().to_string())
                    .await?;
            }
            PrivateCommand::SendMessage(chat_id, msg_string) => {
                bot.send_message(ChatId(chat_id), msg_string).await?;
            }
        }
        Ok(())
    }
}
