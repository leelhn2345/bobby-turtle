use chrono::{Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use tower::util::CallAllUnordered;

#[derive(thiserror::Error, Debug)]
pub enum CalendarError {
    #[error("chrono crate returns no data")]
    None,

    #[error("{0}")]
    Parsing(String),
}

#[tracing::instrument(skip_all)]
/// a string of empty space - ` ` is needed to render keyboard.
pub fn calendar() -> Result<InlineKeyboardMarkup, CalendarError> {
    let now = Utc::now().with_timezone(&Tz::Singapore);

    let curr_day = now.day();
    let curr_month = now.month();
    let curr_year = now.year();

    let weekday_of_first_day = now.with_day(1).ok_or(CalendarError::None)?.weekday();

    let mut calendar_vec = vec![
        Weekday::Mon.to_string(),
        Weekday::Tue.to_string(),
        Weekday::Wed.to_string(),
        Weekday::Thu.to_string(),
        Weekday::Fri.to_string(),
        Weekday::Sat.to_string(),
        Weekday::Sun.to_string(),
    ];

    let empty_space = String::from(" ");

    match weekday_of_first_day {
        Weekday::Mon => calendar_vec.append(&mut vec![empty_space.clone(); 0]),
        Weekday::Tue => calendar_vec.append(&mut vec![empty_space.clone(); 1]),
        Weekday::Wed => calendar_vec.append(&mut vec![empty_space.clone(); 2]),
        Weekday::Thu => calendar_vec.append(&mut vec![empty_space.clone(); 3]),
        Weekday::Fri => calendar_vec.append(&mut vec![empty_space.clone(); 4]),
        Weekday::Sat => calendar_vec.append(&mut vec![empty_space.clone(); 5]),
        Weekday::Sun => calendar_vec.append(&mut vec![empty_space.clone(); 6]),
    };

    let days_passed_in_curr_month = (curr_day - 1)
        .try_into()
        .map_err(|_| CalendarError::Parsing("can't parse into usize".to_string()))?;
    calendar_vec.append(&mut vec![empty_space.clone(); days_passed_in_curr_month]);

    let year = if curr_month == 12 {
        curr_year + 1
    } else {
        curr_year
    };
    let month = if curr_month == 12 { 1 } else { curr_month + 1 };

    let naive_last_day_of_month = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or(CalendarError::None)?
        .pred_opt()
        .ok_or(CalendarError::None)?
        .day();
    let last_day_of_month = now
        .with_day(naive_last_day_of_month)
        .ok_or(CalendarError::None)?
        .day();

    if curr_day != last_day_of_month {
        (curr_day..=last_day_of_month).for_each(|i| calendar_vec.push(i.to_string()));
    }
    let last_weekday_of_month = now
        .with_day(last_day_of_month)
        .ok_or(CalendarError::None)?
        .weekday();

    match last_weekday_of_month {
        Weekday::Mon => calendar_vec.append(&mut vec![empty_space.clone(); 6]),
        Weekday::Tue => calendar_vec.append(&mut vec![empty_space.clone(); 5]),
        Weekday::Wed => calendar_vec.append(&mut vec![empty_space.clone(); 4]),
        Weekday::Thu => calendar_vec.append(&mut vec![empty_space.clone(); 3]),
        Weekday::Fri => calendar_vec.append(&mut vec![empty_space.clone(); 2]),
        Weekday::Sat => calendar_vec.append(&mut vec![empty_space.clone(); 1]),
        Weekday::Sun => calendar_vec.append(&mut vec![empty_space.clone(); 0]),
    }

    let mut keyboard_buttons: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    for week in calendar_vec.chunks(7) {
        let week_row = week
            .iter()
            .map(|week| InlineKeyboardButton::callback(week.to_owned(), week.to_owned()))
            .collect();

        keyboard_buttons.push(week_row);
    }

    tracing::debug!(?keyboard_buttons);
    Ok(InlineKeyboardMarkup::new(keyboard_buttons))
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use chrono::{Datelike, NaiveDate};

    use super::CalendarError;

    #[test]
    fn calendar() -> Result<(), CalendarError> {
        let now = Instant::now();
        let res = super::calendar()?;
        let elapsed = now.elapsed();
        println!("{res:?}");
        println!("{elapsed:?}");
        Ok(())
    }
    #[test]
    fn next_year() {
        let naive_last_day_of_month = NaiveDate::from_ymd_opt(1, 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
            .day();
        println!("{naive_last_day_of_month}");
    }
}
