use teloxide::{
    requests::{Requester, ResponseResult},
    types::{InputFile, Message},
    Bot,
};

pub async fn send_sticker(bot: &Bot, msg: &Message, sticker_id: String) -> ResponseResult<()> {
    bot.send_sticker(msg.chat.id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}

pub async fn send_many_stickers(
    bot: &Bot,
    msg: &Message,
    sticker_ids: Vec<String>,
) -> ResponseResult<()> {
    for sticker in sticker_ids {
        bot.send_sticker(msg.chat.id, InputFile::file_id(sticker))
            .await?;
    }
    Ok(())
}
