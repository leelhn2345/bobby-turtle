use crate::settings::Settings;
use crate::types::{DpHandler, DpHandlerResult};
use crate::utils::stickers::{send_many_stickers, send_sticker};
use teloxide::dptree;
use teloxide::requests::Requester;
use teloxide::{dispatching::HandlerExt, types::Message, utils::command::BotCommands, Bot};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "hello master 🐢 😊")]
enum BotCommand {
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

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommand {
    Start,
}

async fn parse_command(
    bot: Bot,
    msg: Message,
    settings: Settings,
    cmd: BotCommand,
) -> DpHandlerResult {
    match cmd {
        BotCommand::Help => {
            bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
                .await?;
        }
        BotCommand::Hug => send_sticker(&bot, &msg, settings.stickers.hug).await?,
        BotCommand::Kiss => send_sticker(&bot, &msg, settings.stickers.kiss).await?,
        BotCommand::Party => {
            send_many_stickers(&bot, &msg, settings.stickers.party_animals).await?
        }
        _ => {
            bot.send_message(msg.chat.id, "~ feature coming soon ~")
                .await?;
            send_sticker(&bot, &msg, settings.stickers.coming_soon).await?;
        }
    }
    Ok(())
}

async fn admin_cmd_handler(
    bot: Bot,
    msg: Message,
    settings: Settings,
    cmd: AdminCommand,
) -> DpHandlerResult {
    match cmd {
        AdminCommand::Start => {
            let text = match msg.chat.username() {
                Some(x) => format!("hello @{}! 🐢", x),
                None => String::from("hello friend!"),
            };
            bot.send_message(msg.chat.id, text).await?;
            send_sticker(&bot, &msg, settings.stickers.hello).await?;
        }
    }
    Ok(())
}

pub fn bot_command_handler() -> DpHandler {
    dptree::entry()
        .branch(
            dptree::entry()
                .filter_command::<BotCommand>()
                .endpoint(parse_command),
        )
        .branch(
            dptree::entry()
                .filter_command::<AdminCommand>()
                .endpoint(admin_cmd_handler),
        )
}
