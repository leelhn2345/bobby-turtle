use std::collections::HashSet;

use censor::Censor::{self, Custom, Standard, Zealous};
use once_cell::sync::Lazy;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::Message, Bot};

use crate::bot::{check_is_owner, BOT_ME};
use crate::{settings::Settings, stickers::send_sticker, types::MyResult};

static VULGARITIES: Lazy<Censor> = Lazy::new(|| {
    let custom_words: Vec<&str> = vec!["knn", "ccb", "wtf", "wtfbbq", "omfg", "kpkb"];

    let set: HashSet<String> = custom_words.into_iter().map(ToString::to_string).collect();

    Standard + Zealous + Custom(set) - "hell"
});

/// this filter doesnt work on self
pub fn check_vulgar(msg: Message) -> bool {
    let Some(user) = msg.from() else { return false };

    if user.username == BOT_ME.username || check_is_owner(msg.clone()) {
        return false;
    }
    VULGARITIES.check(msg.text().unwrap_or_default())
}

#[tracing::instrument(skip_all)]
pub async fn scold_vulgar_message(bot: Bot, msg: Message, settings: Settings) -> MyResult<()> {
    send_sticker(&bot, &msg.chat.id, settings.stickers.angry).await?;
    bot.send_message(msg.chat.id, "no vulgarities! 😡")
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
