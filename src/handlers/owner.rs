use crate::{
    settings::Settings,
    stickers::{send_many_stickers, send_sticker},
    types::MyResult,
    utils::datetime_now,
};
use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

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
    #[command(description = "the current date & time")]
    Now,
}
impl OwnerCommand {
    pub async fn parse_group_commands(
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
            OwnerCommand::Hug => send_sticker(&bot, &msg.chat.id, settings.stickers.hug).await?,
            OwnerCommand::Kiss => send_sticker(&bot, &msg.chat.id, settings.stickers.kiss).await?,
            OwnerCommand::Party => {
                send_many_stickers(&bot, &msg.chat.id, settings.stickers.party_animals).await?
            }
            OwnerCommand::Love => send_sticker(&bot, &msg.chat.id, settings.stickers.love).await?,

            OwnerCommand::Feed => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }
            OwnerCommand::Now => {
                bot.send_message(msg.chat.id, datetime_now()).await?;
            }
        }
        Ok(())
    }
    pub async fn parse_private_commands() {}
}
