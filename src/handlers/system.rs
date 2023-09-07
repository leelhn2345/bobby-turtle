use teloxide::{requests::Requester, types::Message, Bot};

use crate::{settings::Settings, stickers::send_sticker, types::MyResult};

pub async fn handle_new_member(bot: Bot, msg: Message, settings: Settings) -> MyResult<()> {
    let Some(new_members) = msg.new_chat_members() else {
        return Ok(());
    };
    let bot_details = bot.get_me().await?;
    let bot_username = bot_details.username.as_ref().unwrap();
    let bot_name = &bot_details.first_name;
    for member in new_members {
        let text = match &member.username {
            Some(x) => {
                if bot_username == x {
                    format!("Hello everyone!! I'm {}! 🐢.", bot_name)
                } else {
                    format!("hello @{}", x)
                }
            }
            None => format!("hello {}", member.first_name),
        };

        bot.send_message(msg.chat.id, text)
            // .reply_to_message_id(msg.id)
            .await?;
    }
    send_sticker(&bot, &msg, settings.stickers.hello).await?;
    Ok(())
}

pub async fn handle_left_member(bot: &Bot, msg: &Message, settings: Settings) -> MyResult<()> {
    let Some(member) = msg.left_chat_member() else {
        return Ok(());
    };
    let text = format!("sayonara {} ~~ 😭😭😭", member.full_name());
    bot.send_message(msg.chat.id, text).await?;
    send_sticker(bot, msg, settings.stickers.sad).await?;
    Ok(())
}
