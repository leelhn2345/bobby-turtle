use anyhow::bail;
use chrono::{Datelike, NaiveDate};
use teloxide::{
    payloads::EditMessageTextSetters,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    Bot,
};

use super::{
    calendar::{calendar, DATE_PICK_MSG},
    CallbackDialogue, CallbackState,
};

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
            self.minute += 1;
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn time_pick_keyboard(
    tenth_hour: u8,
    hour: u8,
    tenth_minute: u8,
    minute: u8,
) -> InlineKeyboardMarkup {
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

#[allow(clippy::cast_possible_wrap)]
#[tracing::instrument(skip_all)]
pub async fn time_pick_callback(
    bot: Bot,
    q: CallbackQuery,
    callback: CallbackDialogue,
    (naive_date, remind_time): (NaiveDate, RemindTime),
) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id).await?;

    let Some(data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        bail!("no callback query data")
    };
    let Some(msg) = q.message else {
        tracing::error!("no message data from telegram");
        bail!("no telegram message data")
    };

    let Message { chat, id, .. } = &msg;

    let Some(text) = msg.text() else {
        tracing::error!("no message text from telegram");
        bail!("no telegram message text");
    };

    if data.trim().is_empty() {
        return Ok(());
    }

    let chosen_day = naive_date.day0() + 1;
    let chosen_month = naive_date.month0() + 1;
    let chosen_year = naive_date.year_ce().1 as i32;

    if data == BACK {
        let calendar = calendar(chosen_day, chosen_month, chosen_year).map_err(|e| {
            tracing::error!("{e:#?}");
            e
        })?;
        bot.edit_message_text(chat.id, *id, DATE_PICK_MSG)
            .reply_markup(calendar)
            .await?;
    } else if data == NEXT {
        // TODO:convert remindtime to naivetime
    } else {
        let time_select: TimeSelect = match data.try_into() {
            Ok(x) => x,
            Err(e) => {
                tracing::error!(e);
                bail!("can't parse data into TimeSelect");
            }
        };
        let mut remind_time = remind_time;
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

        tracing::debug!("{remind_time:#?}");

        callback
            .update(CallbackState::RemindDateTime {
                date: naive_date,
                time: remind_time.clone(),
            })
            .await?;

        let time_pick = time_pick_keyboard(
            remind_time.tenth_hour,
            remind_time.hour,
            remind_time.tenth_minute,
            remind_time.minute,
        );

        bot.edit_message_text(chat.id, *id, text)
            .reply_markup(time_pick)
            .await?;
    }
    Ok(())
}
