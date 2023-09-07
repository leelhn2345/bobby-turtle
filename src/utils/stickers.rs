use teloxide::{
    requests::{Requester, ResponseResult},
    types::{InputFile, Message},
    Bot,
};

pub async fn sticker_kiss(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    bot.send_sticker(
        msg.chat.id,
        InputFile::file_id(
            "CAACAgIAAxkBAAEln_Fk9KrjpXTWYeLi-xynbyw1Kl_wPAACBAIAAhZCawqh06yi3GPURTAE",
        ),
    )
    .await?;
    Ok(())
}
