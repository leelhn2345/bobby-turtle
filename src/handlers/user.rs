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
    description = "hello user 🐢😊~. \nThese are the available commands."
)]
pub enum UserCommand {
    #[command(description = "list down all commands")]
    Help,
    #[command(description = "IT'S PARTY TIME!! 🥳🥳")]
    Party,
    #[command(description = "feed me!")]
    Feed,
    #[command(description = "the current date & time")]
    Now,
}

impl UserCommand {
    pub async fn parse_group_commands(
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
                send_many_stickers(&bot, &msg.chat.id, settings.stickers.party_animals).await?;
            }

            UserCommand::Feed => {
                send_sticker(&bot, &msg.chat.id, settings.stickers.coming_soon).await?;
                bot.send_message(msg.chat.id, "~ feature coming soon ~")
                    .await?;
            }

            UserCommand::Now => {
                bot.send_message(msg.chat.id, datetime_now()).await?;
            }
        }
        Ok(())
    }
    pub async fn parse_private_commands() {}
}
