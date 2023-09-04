use teloxide::dptree;
use teloxide::requests::Requester;
use teloxide::{
    dispatching::{HandlerExt, UpdateHandler},
    types::Message,
    utils::command::BotCommands,
    Bot,
};

use super::stickers::{sticker_coming_soon, sticker_kiss};

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
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn parse_command(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Kiss => sticker_kiss(bot, msg).await?,
        _ => sticker_coming_soon(bot, msg).await?,
    }
    Ok(())
}

pub fn bot_command_handler() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    dptree::entry()
        .filter_command::<Command>()
        .endpoint(parse_command)
}
