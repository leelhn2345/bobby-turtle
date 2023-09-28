use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

use crate::types::MyResult;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "🐢 ~ hello admin! ~ 🐢.")]
pub enum AdminCommand {
    #[command(prefix = "/!")]
    Help,
}

impl AdminCommand {
    pub async fn parse_commands(bot: Bot, msg: Message, cmd: AdminCommand) -> MyResult<()> {
        match cmd {
            AdminCommand::Help => {
                bot.send_message(msg.chat.id, AdminCommand::descriptions().to_string())
                    .await?;
            }
        }
        Ok(())
    }
}
