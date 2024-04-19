use anyhow::{Ok, Result};

use sqlx::PgPool;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, User},
    Bot,
};

use crate::{bot::send_sticker, settings::stickers::Stickers};

use super::{chatroom::ChatRoom, BOT_ME};

#[tracing::instrument(name = "bot got added", skip_all)]
pub fn i_got_added(msg: Message) -> bool {
    let new_user = msg.new_chat_members();
    let Some(user) = new_user else { return false };

    if user[0].id == BOT_ME.get().unwrap().id {
        tracing::debug!("i got added");
        true
    } else {
        false
    }
}

#[tracing::instrument(name = "bot got removed", skip_all)]
pub fn i_got_removed(msg: Message) -> bool {
    let old_user = msg.left_chat_member();
    let Some(user) = old_user else { return false };

    if user.id == BOT_ME.get().unwrap().id {
        tracing::debug!("i got removed");
        true
    } else {
        false
    }
}

#[tracing::instrument(name = "im joining", skip_all)]
pub async fn handle_me_join(
    bot: Bot,
    msg: Message,
    pool: PgPool,
    stickers: Stickers,
) -> Result<()> {
    let chat_room = ChatRoom::new(&msg);
    chat_room.save(&pool).await.map_err(|e| {
        tracing::error!("{e:#?}");
        e
    })?;
    let bot_name = &BOT_ME.get().unwrap().first_name;
    let greet = format!("Hello everyone!! I'm {bot_name}!");
    send_sticker(&bot, &msg.chat.id, stickers.hello).await?;
    bot.send_message(msg.chat.id, greet).await?;
    Ok(())
}

#[tracing::instrument(name = "i left", skip_all)]
pub async fn handle_me_left(msg: Message, pool: PgPool) -> Result<()> {
    ChatRoom::leave(&pool, msg.chat.id.0).await.map_err(|e| {
        tracing::error!("{e:#?}");
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "new member", skip_all)]
#[allow(clippy::cast_possible_wrap)]
pub async fn handle_member_join(bot: Bot, msg: Message, stickers: Stickers) -> Result<()> {
    let new_users: Option<Vec<User>> = msg
        .new_chat_members()
        .map(std::borrow::ToOwned::to_owned)
        .map(|users| users.into_iter().filter(|user| !user.is_bot).collect());

    tracing::debug!("{:#?}", new_users);

    let Some(users) = new_users else {
        return Ok(());
    };

    if users.is_empty() {
        return Ok(());
    };

    for user in users {
        tokio::task::spawn({
            let bot = bot.clone();
            async move {
                let text = if let Some(x) = user.username {
                    format!("Hello @{x}!")
                } else {
                    format!("Hello {}!", user.first_name)
                };
                bot.send_message(msg.chat.id, text)
                    .reply_to_message_id(msg.id)
                    .await?;
                Ok(())
            }
        });
    }
    send_sticker(&bot, &msg.chat.id, stickers.hello).await?;

    Ok(())
}

#[tracing::instrument(name = "member left", skip_all)]
pub async fn handle_member_leave(bot: Bot, msg: Message, stickers: Stickers) -> Result<()> {
    let Some(member) = msg.left_chat_member() else {
        return Ok(());
    };
    let text = format!("Sayanora {} ~~ 😭😭😭", member.full_name());
    send_sticker(&bot, &msg.chat.id, stickers.sad).await?;
    bot.send_message(msg.chat.id, text)
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
