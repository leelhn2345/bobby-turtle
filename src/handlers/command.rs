use crate::{
    settings::Settings,
    stickers::{send_many_stickers, send_sticker},
    types::MyResult,
};
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "hello owner!~ 😊")]
pub enum OwnerCommand {
    #[command(description = "list down all commands")]
    Help,
    #[command(description = "a lovely hug! 🤗❤️")]
    Hug,
    // #[command(description = "greetings")]
    // Greet,
    #[command(description = "a passionate kiss")]
    Kiss,
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
            _ => {
                send_sticker(&bot, &msg, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "hello user 😊~")]
pub enum UserCommand {
    Start,
}

impl UserCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: UserCommand,
    ) -> MyResult<()> {
        match cmd {
            UserCommand::Start => {
                let text = match msg.chat.username() {
                    Some(x) => format!("hello @{}! 🐢", x),
                    None => String::from("hello friend!"),
                };
                bot.send_message(msg.chat.id, text).await?;
                send_sticker(&bot, &msg, settings.stickers.hello).await?
            }
        }
        Ok(())
    }
}
