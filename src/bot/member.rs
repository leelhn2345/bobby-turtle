use anyhow::{Ok, Result};
use teloxide::{requests::Requester, types::Message, Bot};

pub async fn handle_member_join(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, "hello").await?;
    Ok(())
}
pub async fn handle_member_leave(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, "byebye").await?;
    Ok(())
}
