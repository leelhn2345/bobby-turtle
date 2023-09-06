use teloxide::{dispatching::MessageFilterExt, dptree, requests::Requester, types::Message, Bot};

use crate::types::{DpHandler, DpHandlerResult};

use super::stickers::{sticker_hello, sticker_sad};

pub fn bot_message_handler() -> DpHandler {
    // let message_handler = Update::filter_message;
    // dptree::filter(|msg: Message| {})
    dptree::entry()
        .branch(Message::filter_new_chat_members().endpoint(handle_new_member))
        .branch(Message::filter_left_chat_member().endpoint(handle_left_member))
}

async fn handle_new_member(bot: Bot, msg: Message) -> DpHandlerResult {
    let Some(new_members) = msg.new_chat_members() else {
        return Ok(());
    };
    for member in new_members {
        let text = match &member.username {
            Some(x) => {
                if &bot.get_me().await?.user.username.unwrap() == x {
                    "Hello everyone!! I'm Bobby! 🐢".to_string()
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
    sticker_hello(bot, msg).await?;
    Ok(())
}

async fn handle_left_member(bot: Bot, msg: Message) -> DpHandlerResult {
    let Some(member) = msg.left_chat_member() else {
        return Ok(());
    };
    let text = format!("sayonara {} ~~ 😭😭😭", member.full_name());
    bot.send_message(msg.chat.id, text).await?;
    sticker_sad(bot, msg).await?;
    Ok(())
}
