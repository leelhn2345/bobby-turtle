use anyhow::bail;
use teloxide::{
    payloads::EditMessageTextSetters,
    requests::Requester,
    types::{
        CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
    },
    Bot,
};
use time::{
    macros::{format_description, offset},
    Date, OffsetDateTime,
};

use crate::callbacks::expired_callback_msg;

use super::{date_page, remind_text_page, CallbackPage, CallbackState};

const BACK: &str = "Back";
const NEXT: &str = "Next";

const TEN_HOUR_UP: &str = "TenHourUp";
const HOUR_UP: &str = "HourUp";
const TEN_MINUTE_UP: &str = "TenMinuteUp";
const MINUTE_UP: &str = "MinuteUp";

const TEN_HOUR_DOWN: &str = "TenHourDown";
const HOUR_DOWN: &str = "HourDown";
const TEN_MINUTE_DOWN: &str = "TenMinuteDown";
const MINUTE_DOWN: &str = "MinuteDown";

enum TimeSelect {
    TenHourUp,
    HourUp,
    TenMinuteUp,
    MinuteUp,

    TenHourDown,
    HourDown,
    TenMinuteDown,
    MinuteDown,
}

impl TimeSelect {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeSelect::TenHourUp => TEN_HOUR_UP,
            TimeSelect::HourUp => HOUR_UP,
            TimeSelect::TenMinuteUp => TEN_MINUTE_UP,
            TimeSelect::MinuteUp => MINUTE_UP,

            TimeSelect::TenHourDown => TEN_HOUR_DOWN,
            TimeSelect::HourDown => HOUR_DOWN,
            TimeSelect::TenMinuteDown => TEN_MINUTE_DOWN,
            TimeSelect::MinuteDown => MINUTE_DOWN,
        }
    }
}

impl TryFrom<String> for TimeSelect {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            TEN_HOUR_UP => Ok(Self::TenHourUp),
            TEN_HOUR_DOWN => Ok(Self::TenHourDown),
            HOUR_UP => Ok(Self::HourUp),
            HOUR_DOWN => Ok(Self::HourDown),

            TEN_MINUTE_UP => Ok(Self::TenMinuteUp),
            TEN_MINUTE_DOWN => Ok(Self::TenMinuteDown),
            MINUTE_UP => Ok(Self::MinuteUp),
            MINUTE_DOWN => Ok(Self::MinuteDown),

            unknown => Err(format!("{unknown} is unsupported.")),
        }
    }
}
#[derive(Clone, Debug)]
pub struct RemindTime {
    pub tenth_hour: u8,
    pub hour: u8,
    pub tenth_minute: u8,
    pub minute: u8,
}

impl Default for RemindTime {
    fn default() -> Self {
        Self {
            tenth_hour: 1,
            hour: 2,
            tenth_minute: 0,
            minute: 0,
        }
    }
}

impl RemindTime {
    fn new(hour: u8, minute: u8) -> Result<Self, String> {
        if hour > 23 {
            return Err(format!("invalid hour: {hour}"));
        }

        if minute > 59 {
            return Err(format!("invalid minuet: {minute}"));
        }

        let tenth_hour = hour / 10;
        let hour = hour % 10;
        let tenth_minute = minute / 10;
        let minute = minute % 10;

        Ok(Self {
            tenth_hour,
            hour,
            tenth_minute,
            minute,
        })
    }

    fn tenth_hour_up(&mut self) {
        match self.hour {
            0..=3 => {
                if self.tenth_hour >= 2 {
                    self.tenth_hour = 0;
                } else {
                    self.tenth_hour += 1;
                }
            }
            4.. => {
                if self.tenth_hour >= 1 {
                    self.tenth_hour = 0;
                } else {
                    self.tenth_hour += 1;
                }
            }
        }
    }

    fn tenth_hour_down(&mut self) {
        match self.tenth_hour {
            1.. => self.tenth_hour -= 1,
            0 => match self.hour {
                0..=3 => self.tenth_hour = 2,
                4.. => self.tenth_hour = 1,
            },
        }
    }

    fn hour_up(&mut self) {
        match self.tenth_hour {
            0 | 1 => {
                if self.hour >= 9 {
                    self.hour = 0;
                    self.tenth_hour += 1;
                } else {
                    self.hour += 1;
                }
            }
            2.. => {
                if self.hour >= 3 {
                    self.tenth_hour = 0;
                    self.hour = 0;
                } else {
                    self.hour += 1;
                }
            }
        }
    }

    fn hour_down(&mut self) {
        match self.tenth_hour {
            1.. => {
                if self.hour == 0 {
                    self.tenth_hour -= 1;
                    self.hour = 9;
                } else {
                    self.hour -= 1;
                }
            }
            0 => {
                if self.hour == 0 {
                    self.tenth_hour = 2;
                    self.hour = 3;
                } else {
                    self.hour -= 1;
                }
            }
        }
    }

    fn tenth_minute_up(&mut self) {
        if self.tenth_minute >= 5 {
            self.tenth_minute = 0;
        } else {
            self.tenth_minute += 1;
        }
    }
    fn tenth_minute_down(&mut self) {
        if self.tenth_minute == 0 {
            self.tenth_minute = 5;
        } else {
            self.tenth_minute -= 1;
        }
    }

    fn minute_up(&mut self) {
        if self.minute == 9 {
            self.minute = 0;
        } else {
            self.minute += 1;
        }
    }

    fn minute_down(&mut self) {
        if self.minute == 0 {
            self.minute = 9;
        } else {
            self.minute -= 1;
        }
    }
}

pub async fn time_page(
    bot: Bot,
    chat_id: ChatId,
    msg_id: MessageId,
    naive_date: Date,
    remind_time: RemindTime,
) -> anyhow::Result<()> {
    let month = naive_date.month();
    let day = naive_date.day();
    let year = naive_date.year();
    let text = format!(
        r"You have chosen: 

year: {year}
month: {month} 
day: {day}

Now, let's choose the time. 🐢
The time is in 24 hours format."
    );

    let time_pick = time_keyboard(
        remind_time.tenth_hour,
        remind_time.hour,
        remind_time.tenth_minute,
        remind_time.minute,
    );

    bot.edit_message_text(chat_id, msg_id, text)
        .reply_markup(time_pick)
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
fn time_keyboard(tenth_hour: u8, hour: u8, tenth_minute: u8, minute: u8) -> InlineKeyboardMarkup {
    let up_arrow: &str = "↑";

    tracing::debug!(?tenth_hour);
    tracing::debug!(?hour);
    tracing::debug!(?tenth_minute);
    tracing::debug!(?minute);

    let tenth_hour = tenth_hour.to_string();
    let hour = hour.to_string();
    let tenth_minute = tenth_minute.to_string();
    let minute = minute.to_string();

    let up_btn_row: Vec<InlineKeyboardButton> = vec![
        InlineKeyboardButton::callback(up_arrow, TimeSelect::TenHourUp.as_str()),
        InlineKeyboardButton::callback(up_arrow, TimeSelect::HourUp.as_str()),
        InlineKeyboardButton::callback(up_arrow, TimeSelect::TenMinuteUp.as_str()),
        InlineKeyboardButton::callback(up_arrow, TimeSelect::MinuteUp.as_str()),
    ];
    let time_row: Vec<InlineKeyboardButton> = vec![
        InlineKeyboardButton::callback(tenth_hour, " "),
        InlineKeyboardButton::callback(hour, " "),
        InlineKeyboardButton::callback(tenth_minute, " "),
        InlineKeyboardButton::callback(minute, " "),
    ];

    let down_arrow: &str = "↓";
    let down_btn_row: Vec<InlineKeyboardButton> = vec![
        InlineKeyboardButton::callback(down_arrow, TimeSelect::TenHourDown.as_str()),
        InlineKeyboardButton::callback(down_arrow, TimeSelect::HourDown.as_str()),
        InlineKeyboardButton::callback(down_arrow, TimeSelect::TenMinuteDown.as_str()),
        InlineKeyboardButton::callback(down_arrow, TimeSelect::MinuteDown.as_str()),
    ];

    let last_row: Vec<InlineKeyboardButton> = vec![
        InlineKeyboardButton::callback(BACK, BACK),
        InlineKeyboardButton::callback(NEXT, NEXT),
    ];
    let keyboard: Vec<Vec<InlineKeyboardButton>> =
        vec![up_btn_row, time_row, down_btn_row, last_row];

    InlineKeyboardMarkup::new(keyboard)
}

#[tracing::instrument(skip_all)]
pub async fn time_callback(
    bot: Bot,
    q: CallbackQuery,
    p: CallbackState,
    (naive_date, remind_time): (Date, RemindTime),
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;
    let Some(ref data) = q.data else {
        tracing::error!("query data is None. should contain string or empty spaces.");
        bail!("no callback query data")
    };
    let Some(msg) = q.regular_message() else {
        tracing::error!("no message data from telegram");
        bail!("no telegram message data")
    };

    let Message {
        chat, id: msg_id, ..
    } = &msg;

    if data.trim().is_empty() {
        return Ok(());
    }

    let now = OffsetDateTime::now_utc().to_offset(offset!(+8));

    match data.as_ref() {
        BACK => {
            p.update(CallbackPage::RemindDate).await?;
            date_page(
                bot,
                chat.id,
                *msg_id,
                now.day(),
                now.month().into(),
                now.year(),
            )
            .await?;
        }
        NEXT => {
            tracing::debug!("Next is pressed");
            let hour = remind_time.tenth_hour * 10 + remind_time.hour;
            let minute = remind_time.tenth_minute * 10 + remind_time.minute;

            let naive_datetime = naive_date.with_hms(hour, minute, 0)?;
            tracing::debug!("{naive_datetime:#?}");

            let chosen_datetime = naive_datetime
                .assume_utc()
                .to_offset(offset!(+8))
                .replace_year(naive_datetime.year())
                .map_err(|e| {
                    tracing::debug!("year");
                    e
                })?
                .replace_month(naive_datetime.month())
                .map_err(|e| {
                    tracing::debug!("month");
                    e
                })?
                .replace_day(naive_datetime.day())
                .map_err(|e| {
                    tracing::debug!("day");
                    e
                })?
                .replace_hour(hour)
                .map_err(|e| {
                    tracing::debug!("hour");
                    e
                })?
                .replace_minute(minute)
                .map_err(|e| {
                    tracing::debug!("minute");
                    e
                })?;

            tracing::debug!("{chosen_datetime:#?}");

            time_check(&bot, chat.id, chosen_datetime, now).await?;
            tracing::debug!("time has been checked");

            p.update(CallbackPage::ConfirmDateTime {
                date_time: chosen_datetime,
            })
            .await?;

            remind_text_page(bot, chat.id, *msg_id, chosen_datetime).await?;
        }
        _ => {
            let mut remind_time = remind_time;
            let time_select: TimeSelect = match data.clone().try_into() {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!(e);
                    expired_callback_msg(bot, chat.id, *msg_id).await?;
                    bail!("can't parse data into TimeSelect");
                }
            };
            match time_select {
                TimeSelect::TenHourUp => remind_time.tenth_hour_up(),
                TimeSelect::HourUp => remind_time.hour_up(),
                TimeSelect::TenMinuteUp => remind_time.tenth_minute_up(),
                TimeSelect::MinuteUp => remind_time.minute_up(),
                TimeSelect::TenHourDown => remind_time.tenth_hour_down(),
                TimeSelect::HourDown => remind_time.hour_down(),
                TimeSelect::TenMinuteDown => remind_time.tenth_minute_down(),
                TimeSelect::MinuteDown => remind_time.minute_down(),
            };

            p.update(CallbackPage::RemindDateTime {
                date: naive_date,
                time: remind_time.clone(),
            })
            .await?;

            time_page(bot, chat.id, *msg_id, naive_date, remind_time).await?;
        }
    }
    Ok(())
}

pub async fn change_time_callback(
    bot: Bot,
    q: CallbackQuery,
    p: CallbackState,
    date_time: OffsetDateTime,
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id.clone()).await?;

    let Some(Message {
        id: msg_id, chat, ..
    }) = q.regular_message()
    else {
        tracing::error!("no message data from telegram");
        bail!("no query message data");
    };
    let chosen_hour = date_time.hour();
    let chosen_minute = date_time.minute();

    let Ok(remind_time) = RemindTime::new(chosen_hour, chosen_minute) else {
        bail!("can't parse datetime");
    };

    let naive_date = date_time.date();

    p.update(CallbackPage::RemindDateTime {
        date: naive_date,
        time: remind_time.clone(),
    })
    .await?;

    time_page(bot, chat.id, *msg_id, naive_date, remind_time).await?;
    Ok(())
}

pub async fn time_check(
    bot: &Bot,
    chat_id: ChatId,
    chosen_datetime: OffsetDateTime,
    now: OffsetDateTime,
) -> anyhow::Result<()> {
    if chosen_datetime < now {
        tracing::error!("chosen datetime is in the past");
        let format = format_description!("[hour]:[minute]:[second]");
        let current_time = now.time().format(&format)?;
        let text = format!(
            r"You can't send a message into the past. ❌

Messages should be after this instant.
The current time is {current_time}."
        );
        bot.send_message(chat_id, text).await?;

        bail!("chosen datetime can't be before this current instant");
    }
    Ok(())
}
