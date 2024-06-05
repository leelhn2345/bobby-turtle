use anyhow::bail;
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;
use teloxide::{
    payloads::EditMessageTextSetters,
    requests::Requester,
    types::{
        CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId,
    },
    Bot,
};

use crate::callbacks::expired_callback_msg;

use super::{occurence_page, time::RemindTime, time_page, CallbackPage, CallbackState};

const CURRENT_MONTH: &str = "Current";
const OCCURENCE: &str = "Occurence";
const DATE_PICK_MSG: &str = "Pick your date 🐢";

#[derive(thiserror::Error, Debug)]
pub enum DateError {
    #[error("chrono crate returns no data")]
    None,

    #[error("{0}")]
    Parse(String),

    #[error("Invalid data chosen")]
    InvalidData,

    #[error("wrong era")]
    WrongEra,

    #[error("No callback data")]
    NoCallbackData,

    #[error("No message data from telegram")]
    NoMessageData,
}

pub async fn date_page(
    bot: Bot,
    chat_id: ChatId,
    msg_id: MessageId,
    day: u32,
    month: u32,
    year: i32,
) -> anyhow::Result<()> {
    let keyboard = date_keyboard(day, month, year)?;
    bot.edit_message_text(chat_id, msg_id, DATE_PICK_MSG)
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

fn date_keyboard(day: u32, month: u32, year: i32) -> Result<InlineKeyboardMarkup, DateError> {
    let now = Utc::now().with_timezone(&Tz::Singapore);

    let then = now
        .with_year(year)
        .ok_or(DateError::None)?
        .with_month(month)
        .ok_or(DateError::None)?;

    // the earliest possible date is today, hence if `then` > `now`,
    // change to specified date
    if then > now {
        then.with_day(day).ok_or(DateError::None)?;
    }

    let mut calendar_vec: Vec<InlineKeyboardButton> = Vec::new();

    let weekday_of_first_day = then.with_day(1).ok_or(DateError::None)?.weekday();

    match weekday_of_first_day {
        Weekday::Mon => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 0]),
        Weekday::Tue => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 1]),
        Weekday::Wed => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 2]),
        Weekday::Thu => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 3]),
        Weekday::Fri => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 4]),
        Weekday::Sat => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 5]),
        Weekday::Sun => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 6]),
    };

    let days_passed_in_curr_month = (day - 1)
        .try_into()
        .map_err(|_| DateError::Parse("can't parse into usize".to_string()))?;

    calendar_vec.append(&mut vec![
        InlineKeyboardButton::callback(" ", " ");
        days_passed_in_curr_month
    ]);

    let past_future_month_year = get_past_future_month_year(month, year);

    let PastFutureMonthYear {
        next_month,
        year_of_next_month,
        ..
    } = past_future_month_year;

    let naive_last_day_of_month = NaiveDate::from_ymd_opt(year_of_next_month, next_month, 1)
        .ok_or(DateError::None)?
        .pred_opt()
        .ok_or(DateError::None)?
        .day();
    let last_day_of_month = then
        .with_day(naive_last_day_of_month)
        .ok_or(DateError::None)?
        .day();

    let mut wow = (day..=last_day_of_month)
        .map(|i| InlineKeyboardButton::callback(i.to_string(), format!("{i}-{month}-{year}")))
        .collect();
    calendar_vec.append(&mut wow);

    let last_weekday_of_month = then
        .with_day(last_day_of_month)
        .ok_or(DateError::None)?
        .weekday();

    match last_weekday_of_month {
        Weekday::Mon => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 6]),
        Weekday::Tue => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 5]),
        Weekday::Wed => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 4]),
        Weekday::Thu => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 3]),
        Weekday::Fri => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 2]),
        Weekday::Sat => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 1]),
        Weekday::Sun => calendar_vec.append(&mut vec![InlineKeyboardButton::callback(" ", " "); 0]),
    }

    let mut calendar = date_keyboard_pagination_row(month, year, past_future_month_year, now)?;

    for week in calendar_vec.chunks(7) {
        calendar.push(week.to_owned());
    }
    let mut occurence_row = vec![InlineKeyboardButton::callback("Back", OCCURENCE)];
    if then.month() != now.month() {
        occurence_row.push(InlineKeyboardButton::callback(CURRENT_MONTH, CURRENT_MONTH));
    }
    calendar.push(occurence_row);
    Ok(InlineKeyboardMarkup::new(calendar))
}

fn date_keyboard_pagination_row(
    month: u32,
    year: i32,
    data: PastFutureMonthYear,
    now: DateTime<Tz>,
) -> Result<Vec<Vec<InlineKeyboardButton>>, DateError> {
    let month_name = parse_month_to_str(month)?;
    let calendar_title =
        InlineKeyboardButton::callback(format!("{month_name} {year}"), " ".to_owned());

    let PastFutureMonthYear {
        prev_month,
        year_of_prev_month,
        next_month,
        year_of_next_month,
    } = data;

    let prev_month_first_day = now
        .with_year(year_of_prev_month)
        .ok_or(DateError::None)?
        .with_month(prev_month)
        .ok_or(DateError::None)?
        .with_day(1)
        .ok_or(DateError::None)?;

    let prev_month_calendar = if prev_month_first_day > now {
        let prev_month_date = format!("01-{prev_month}-{year_of_prev_month} <<");
        InlineKeyboardButton::callback("<<", prev_month_date.clone())
    } else {
        let curr_day = now.day();
        let curr_month = now.month();
        let curr_year = now.year();
        if prev_month == curr_month && year_of_prev_month == curr_year {
            let prev_month_date = format!("{curr_day}-{prev_month}-{year_of_prev_month} <<");
            InlineKeyboardButton::callback("<<", prev_month_date.clone())
        } else {
            InlineKeyboardButton::callback(" ", " ")
        }
    };

    let next_month_date = format!(">> 01-{next_month}-{year_of_next_month}");

    let next_month_calendar = InlineKeyboardButton::callback(">>", next_month_date.clone());

    let month_row = Vec::from([prev_month_calendar, calendar_title, next_month_calendar]);

    let weekday_key_val = [
        ("Mon", " "),
        ("Tue", " "),
        ("Wed", " "),
        ("Thu", " "),
        ("Fri", " "),
        ("Sat", " "),
        ("Sun", " "),
    ];
    let weekday_buttons = weekday_key_val.map(|x| InlineKeyboardButton::callback(x.0, x.1));
    let weekday_row = Vec::from(weekday_buttons);

    let keyboard_markup = vec![month_row, weekday_row];
    Ok(keyboard_markup)
}

#[allow(clippy::struct_field_names)]
#[derive(Copy, Clone)]
struct PastFutureMonthYear {
    prev_month: u32,
    year_of_prev_month: i32,
    next_month: u32,
    year_of_next_month: i32,
}

/// Gets the information needed for prev and next month.
/// Needed for pagination of calendar.
fn get_past_future_month_year(month: u32, year: i32) -> PastFutureMonthYear {
    let prev_month = if month <= 1 { 12 } else { month - 1 };
    let year_of_prev_month = if month <= 1 { year - 1 } else { year };
    let next_month = if month >= 12 { 1 } else { month + 1 };
    let year_of_next_month = if month >= 12 { year + 1 } else { year };

    PastFutureMonthYear {
        prev_month,
        year_of_prev_month,
        next_month,
        year_of_next_month,
    }
}

fn parse_month_to_str(month: u32) -> Result<&'static str, DateError> {
    match month {
        1 => Ok("Jan"),
        2 => Ok("Feb"),
        3 => Ok("Mar"),
        4 => Ok("Apr"),
        5 => Ok("May"),
        6 => Ok("Jun"),
        7 => Ok("Jul"),
        8 => Ok("Aug"),
        9 => Ok("Sep"),
        10 => Ok("Oct"),
        11 => Ok("Nov"),
        12 => Ok("Dec"),
        _ => Err(DateError::InvalidData), // Handle invalid month numbers
    }
}

async fn send_prev_or_next_month(
    d: NaiveDate,
    chat_id: ChatId,
    msg_id: MessageId,
    bot: Bot,
) -> anyhow::Result<()> {
    let naive_day = d.day0() + 1;
    let naive_month = d.month0() + 1;
    let ce_year_of_naive_month = d.year_ce();
    let naive_year = i32::from_le_bytes(ce_year_of_naive_month.1.to_le_bytes());
    if !ce_year_of_naive_month.0 {
        tracing::error!("year of wrong era - {}", naive_year);
        return Err(DateError::WrongEra.into());
    }
    let calendar = date_keyboard(naive_day, naive_month, naive_year)?;
    bot.edit_message_text(chat_id, msg_id, DATE_PICK_MSG)
        .reply_markup(calendar)
        .await?;
    Ok(())
}

#[allow(deprecated)]
#[tracing::instrument(skip_all)]
pub async fn date_callback(bot: Bot, q: CallbackQuery, p: CallbackState) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id).await?;

    let Some(data) = q.data else {
        tracing::error!("query data is None. should contain string or empty string.");
        return Err(DateError::NoCallbackData.into());
    };
    let Some(Message { id, chat, .. }) = q.message else {
        tracing::error!("no message data from telegram");
        return Err(DateError::NoMessageData.into());
    };

    if data.trim().is_empty() {
        return Ok(());
    } else if data.strip_suffix(" <<").is_some() {
        let naive_prev_month = NaiveDate::parse_from_str(&data, "%d-%m-%Y <<")?;
        send_prev_or_next_month(naive_prev_month, chat.id, id, bot).await?;
    } else if data.strip_prefix(">> ").is_some() {
        let naive_next_month = NaiveDate::parse_from_str(&data, ">> %d-%m-%Y")?;
        send_prev_or_next_month(naive_next_month, chat.id, id, bot).await?;
    } else if NaiveDate::parse_from_str(&data, "%d-%m-%Y").is_ok() {
        let naive_date = NaiveDate::parse_from_str(&data, "%d-%m-%Y")?;

        let remind_time = RemindTime::default();
        p.update(CallbackPage::RemindDateTime {
            date: naive_date,
            time: remind_time.clone(),
        })
        .await?;

        time_page(bot, chat.id, id, naive_date, remind_time).await?;
    } else {
        match data.as_ref() {
            OCCURENCE => {
                p.update(CallbackPage::Occcurence).await?;
                occurence_page(bot, chat.id, id).await?;
            }
            CURRENT_MONTH => {
                let now = Utc::now().with_timezone(&Tz::Singapore);
                date_page(bot, chat.id, id, now.day(), now.month(), now.year()).await?;
            }
            unknown => {
                tracing::error!(unknown, "unrecognizable value");
                expired_callback_msg(bot, chat.id, id).await?;
                bail!(DateError::InvalidData);
            }
        }
    };

    Ok(())
}
