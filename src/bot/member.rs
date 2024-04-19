use anyhow::{Ok, Result};
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, User},
    Bot,
};

use crate::{bot::send_sticker, settings::stickers::Stickers};

#[tracing::instrument(name = "im joining", skip_all)]
pub async fn handle_me_join() -> Result<()> {
    Ok(())
}

#[tracing::instrument(name = "new member", skip_all)]
#[allow(clippy::cast_possible_wrap)]
pub async fn handle_member_join(
    bot: Bot,
    msg: Message,
    pool: PgPool,
    stickers: Stickers,
) -> Result<()> {
    let new_users: Option<Vec<User>> = msg
        .new_chat_members()
        .map(std::borrow::ToOwned::to_owned)
        .map(|users| users.into_iter().filter(|user| !user.is_bot).collect());

    tracing::debug!("{:#?}", new_users);

    if let Some(users) = new_users {
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO users 
            (id, first_name, last_name, username, role, joined_at)",
        );

        query_builder.push_values(&users, |mut b, user| {
            b.push_bind(user.id.0 as i64)
                .push_bind(user.first_name.to_string())
                .push_bind(user.last_name.clone())
                .push_bind(user.username.clone())
                .push_bind("test subject")
                .push_bind(Utc::now());
        });

        let query = query_builder.build();

        query.execute(&pool).await.map_err(|e| {
            tracing::error!("{:#?}", e);
            e
        })?;

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
    }

    Ok(())
}
#[tracing::instrument(name = "member left", skip_all)]
pub async fn handle_member_leave(bot: Bot, msg: Message) -> Result<()> {
    tracing::debug!("someone left");
    bot.send_message(msg.chat.id, "byebye").await?;
    Ok(())
}
