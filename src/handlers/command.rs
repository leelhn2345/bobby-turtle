use crate::{
    settings::Settings,
    stickers::{send_many_stickers, send_sticker},
    types::MyResult,
};
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "hello master 🐢 😊")]
pub enum GeneralCommand {
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
impl GeneralCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: GeneralCommand,
    ) -> MyResult<()> {
        match cmd {
            GeneralCommand::Help => {
                bot.send_message(msg.chat.id, GeneralCommand::descriptions().to_string())
                    .await?;
            }
            GeneralCommand::Hug => send_sticker(&bot, &msg, settings.stickers.hug).await?,
            GeneralCommand::Kiss => send_sticker(&bot, &msg, settings.stickers.kiss).await?,
            GeneralCommand::Party => {
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
#[command(rename_rule = "lowercase")]
pub enum AdminCommand {
    Start,
}

impl AdminCommand {
    pub async fn parse_commands(
        bot: Bot,
        msg: Message,
        settings: Settings,
        cmd: AdminCommand,
    ) -> MyResult<()> {
        match cmd {
            AdminCommand::Start => {
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
