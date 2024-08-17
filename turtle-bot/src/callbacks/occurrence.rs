use anyhow::bail;
use gaia::stickers::Stickers;
use teloxide::{
    payloads::{EditMessageTextSetters, SendMessageSetters},
    requests::Requester,
    types::{
        CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
        ParseMode,
    },
    Bot,
};
use time::{macros::offset, OffsetDateTime};

use crate::{
    callbacks::{date_page, expired_callback_msg, CallbackPage},
    sticker::send_sticker,
};

use super::CallbackState;

const ONE_OFF: &str = "One-Off";
const RECURRING: &str = "Recurring";
pub const OCCURENCE_DESCRIPTION: &str = r"**One-Off** refers to a reminder that only appears once.

**Recurring** refers to a reminder that appears at interval.";

pub enum OccurenceState {
    OneOff,
    Recurring,
}
impl OccurenceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneOff => ONE_OFF,
            Self::Recurring => RECURRING,
        }
    }
}
impl TryFrom<String> for OccurenceState {
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

fn occurence_keyboard() -> InlineKeyboardMarkup {
    let buttons: Vec<Vec<InlineKeyboardButton>> = vec![vec![
        InlineKeyboardButton::callback(
            OccurenceState::OneOff.as_str(),
            OccurenceState::OneOff.as_str(),
        ),
        InlineKeyboardButton::callback(
            OccurenceState::Recurring.as_str(),
            OccurenceState::Recurring.as_str(),
        ),
    ]];

    InlineKeyboardMarkup::new(buttons)
}

#[allow(deprecated)]
#[tracing::instrument(skip_all)]
pub async fn new_occurence_page(bot: Bot, chat_id: ChatId) -> anyhow::Result<()> {
    let keyboard = occurence_keyboard();
    bot.send_message(chat_id, OCCURENCE_DESCRIPTION)
        .parse_mode(ParseMode::Markdown)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

#[allow(deprecated)]
#[tracing::instrument(skip_all)]
pub async fn occurence_page(bot: Bot, chat_id: ChatId, msg_id: MessageId) -> anyhow::Result<()> {
    let keyboard = occurence_keyboard();
    bot.edit_message_text(chat_id, msg_id, OCCURENCE_DESCRIPTION)
        .parse_mode(ParseMode::Markdown)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}
pub async fn occurence_callback(
    bot: Bot,
    q: CallbackQuery,
    p: CallbackState,
    stickers: Stickers,
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    let Some(ref data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        bail!("no query data")
    };
    let Some(Message { id, chat, .. }) = q.regular_message() else {
        tracing::error!("no message data from telegram");
        bail!("no message data")
    };
    let occurence = match OccurenceState::try_from(data.clone()) {
        Err(e) => {
            expired_callback_msg(bot, chat.id, *id).await?;
            bail!("{e}");
        }
        Ok(x) => x,
    };
    match occurence {
        OccurenceState::OneOff => {
            let now = OffsetDateTime::now_utc().to_offset(offset!(+8));
            p.update(CallbackPage::RemindDate).await?;
            tracing::debug!("changed callback state to date");
            date_page(bot, chat.id, *id, now.day(), now.month().into(), now.year()).await?;
        }
        OccurenceState::Recurring => {
            bot.delete_message(chat.id, *id).await?;
            send_sticker(&bot, &chat.id, stickers.coming_soon).await?;
        }
    }
    Ok(())
}
