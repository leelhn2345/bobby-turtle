use anyhow::bail;
use teloxide::{
    requests::Requester,
    types::{CallbackQuery, ChatId, Message, MessageId},
    Bot,
};

#[tracing::instrument(skip_all)]
pub async fn expired_callback(bot: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    let Some(Message { id, chat, .. }) = q.regular_message() else {
        tracing::error!("no message data from telegram");
        bail!("no query message")
    };
    expired_callback_msg(bot, chat.id, *id).await?;
    Ok(())
}

pub async fn expired_callback_msg(
    bot: Bot,
    chat_id: ChatId,
    msg_id: MessageId,
) -> anyhow::Result<()> {
    bot.edit_message_text(chat_id, msg_id, "This has expired 😅 🐢🐢🐢")
        .await?;
    Ok(())
}
