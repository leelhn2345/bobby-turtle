use anyhow::bail;
use chrono::{Datelike, Utc};
use chrono_tz::Tz;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, Chat, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode},
    Bot,
};

use crate::{bot::expired_callback_msg, settings::stickers::Stickers};

use super::{
    calendar::{calendar, DATE_PICK_MSG},
    send_sticker, CallbackDialogue, CallbackState,
};

const ONE_OFF: &str = "One-Off";
const RECURRING: &str = "Recurring";
pub const OCCURENCE_DESCRIPTION: &str = r"**One-Off** refers to a reminder that only appears once.

**Recurring** refers to a reminder that appears at interval.";

pub enum Occurence {
    OneOff,
    Recurring,
}
impl Occurence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneOff => ONE_OFF,
            Self::Recurring => RECURRING,
        }
    }
}
impl TryFrom<String> for Occurence {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            ONE_OFF => Ok(Self::OneOff),
            RECURRING => Ok(Self::Recurring),
            unknown => Err(format!(
                "{unknown} is not a supported occurence value. use either `One-Off` or `Recurring`"
            )),
        }
    }
}

pub fn occurence_keyboard() -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = vec![vec![
        InlineKeyboardButton::callback(Occurence::OneOff.as_str(), Occurence::OneOff.as_str()),
        InlineKeyboardButton::callback(
            Occurence::Recurring.as_str(),
            Occurence::Recurring.as_str(),
        ),
    ]];

    InlineKeyboardMarkup::new(buttons)
}

#[allow(deprecated)]
#[tracing::instrument(skip_all)]
pub async fn pick_occurence(bot: Bot, chat: Chat) -> anyhow::Result<()> {
    let keyboard = occurence_keyboard();
    bot.send_message(chat.id, OCCURENCE_DESCRIPTION)
        .parse_mode(ParseMode::Markdown)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn occurence_callback(
    bot: Bot,
    q: CallbackQuery,
    callback: CallbackDialogue,
    stickers: Stickers,
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id).await?;
    let Some(data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        bail!("no query data")
    };
    let Some(Message { id, chat, .. }) = q.message else {
        tracing::error!("no message data from telegram");
        bail!("no message data")
    };
    let occurence = match Occurence::try_from(data) {
        Err(e) => {
            expired_callback_msg(bot, chat, id).await?;
            bail!("{e}");
        }
        Ok(x) => x,
    };
    match occurence {
        Occurence::OneOff => {
            let now = Utc::now().with_timezone(&Tz::Singapore);
            let calendar = calendar(now.day(), now.month(), now.year())?;
            callback.update(CallbackState::RemindDate).await?;
            tracing::debug!("changed callback state to date");
            bot.edit_message_text(chat.id, id, DATE_PICK_MSG)
                .reply_markup(calendar)
                .await?;
        }
        Occurence::Recurring => {
            bot.delete_message(chat.id, id).await?;
            send_sticker(&bot, &chat.id, stickers.coming_soon).await?;
        }
    };
    Ok(())
}
