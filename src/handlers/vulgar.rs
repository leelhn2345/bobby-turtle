use std::collections::HashSet;

use censor::Censor::{self, Custom, Sex, Standard, Zealous};
use once_cell::sync::Lazy;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::Message, Bot};

use crate::{settings::Settings, stickers::send_sticker, types::MyResult};

static VULGARITIES: Lazy<Censor> = Lazy::new(|| {
    let custom_words: Vec<&str> = vec!["knn", "ccb", "wtf", "wtfbbq", "omfg", "kpkb"];

    let set: HashSet<String> = custom_words.into_iter().map(ToString::to_string).collect();

    Standard + Sex + Zealous + Custom(set)
});

#[tracing::instrument(skip_all)]
pub fn check_vulgar(msg: Message) -> bool {
    VULGARITIES.check(msg.text().unwrap_or_default())
}

#[tracing::instrument(skip_all)]
pub async fn scold_vulgar_message(bot: Bot, msg: Message, settings: Settings) -> MyResult<()> {
    send_sticker(&bot, &msg, settings.stickers.angry).await?;
    bot.send_message(msg.chat.id, "no vulgarities! 😡")
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}
