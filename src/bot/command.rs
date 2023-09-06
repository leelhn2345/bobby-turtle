use teloxide::dptree;
use teloxide::requests::Requester;
use teloxide::{dispatching::HandlerExt, types::Message, utils::command::BotCommands, Bot};

use crate::types::{DpHandler, DpHandlerResult};

use super::stickers::{sticker_coming_soon, sticker_hello, sticker_kiss, sticker_party_animals};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "hello master 🐢 😊")]
enum Command {
    #[command(description = "list down all commands")]
    Help,
    #[command(description = "a hug full of warmth")]
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

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommand {
    Start,
}

async fn parse_command(bot: Bot, msg: Message, cmd: Command) -> DpHandlerResult {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Kiss => sticker_kiss(bot, msg).await?,
        Command::Party => sticker_party_animals(bot, msg).await?,
        _ => {
            bot.send_message(msg.chat.id, "~ feature coming soon ~")
                .await?;
            sticker_coming_soon(bot, msg).await?;
        }
    }
    Ok(())
}

async fn admin_cmd_handler(bot: Bot, msg: Message, cmd: AdminCommand) -> DpHandlerResult {
    match cmd {
        AdminCommand::Start => {
            let text = match msg.chat.username() {
                Some(x) => format!("hello @{}! 🐢", x),
                None => String::from("hello friend!"),
            };
            bot.send_message(msg.chat.id, text).await?;
            sticker_hello(bot, msg).await?;
        }
    }
    Ok(())
}

pub fn bot_command_handler() -> DpHandler {
    dptree::entry()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(parse_command),
        )
        .branch(
            dptree::entry()
                .filter_command::<AdminCommand>()
                .endpoint(admin_cmd_handler),
        )
}
