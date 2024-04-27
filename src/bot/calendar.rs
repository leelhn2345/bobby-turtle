use chrono::{Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

#[derive(thiserror::Error, Debug)]
pub enum CalendarError {
    #[error("chrono crate returns no data")]
    None,

    #[error("{0}")]
    Parsing(String),

    #[error("Invalid data chosen")]
    InvalidData,
}

/// a string of empty space - ` ` is needed to render keyboard.
#[tracing::instrument(skip_all)]
pub fn calendar(day: u32, month: u32, year: i32) -> Result<InlineKeyboardMarkup, CalendarError> {
    let then = Utc::now()
        .with_year(year)
        .ok_or(CalendarError::None)?
        .with_month(month)
        .ok_or(CalendarError::None)?
        .with_day(day)
        .ok_or(CalendarError::None)?
        .with_timezone(&Tz::Singapore);

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

    let weekday_of_first_day = then.with_day(1).ok_or(CalendarError::None)?.weekday();

    match weekday_of_first_day {
        Weekday::Mon => calendar_vec.append(&mut vec![empty_space.clone(); 0]),
        Weekday::Tue => calendar_vec.append(&mut vec![empty_space.clone(); 1]),
        Weekday::Wed => calendar_vec.append(&mut vec![empty_space.clone(); 2]),
        Weekday::Thu => calendar_vec.append(&mut vec![empty_space.clone(); 3]),
        Weekday::Fri => calendar_vec.append(&mut vec![empty_space.clone(); 4]),
        Weekday::Sat => calendar_vec.append(&mut vec![empty_space.clone(); 5]),
        Weekday::Sun => calendar_vec.append(&mut vec![empty_space.clone(); 6]),
    };

    let days_passed_in_curr_month = (day - 1)
        .try_into()
        .map_err(|_| CalendarError::Parsing("can't parse into usize".to_string()))?;
    calendar_vec.append(&mut vec![empty_space.clone(); days_passed_in_curr_month]);

    let next_month = if month >= 12 { 1 } else { month + 1 };
    let year_of_next_month = if month >= 12 { year + 1 } else { year };

    let naive_last_day_of_month = NaiveDate::from_ymd_opt(year_of_next_month, next_month, 1)
        .ok_or(CalendarError::None)?
        .pred_opt()
        .ok_or(CalendarError::None)?
        .day();
    let last_day_of_month = then
        .with_day(naive_last_day_of_month)
        .ok_or(CalendarError::None)?
        .day();

    if day != last_day_of_month {
        (day..=last_day_of_month).for_each(|i| calendar_vec.push(i.to_string()));
    }
    let last_weekday_of_month = then
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

    let month_name = match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => return Err(CalendarError::InvalidData), // Handle invalid month numbers
    };

    let month_row = ["<<", &format!("{month_name} {year}"), ">>"]
        .into_iter()
        .map(|x| InlineKeyboardButton::callback(x.to_owned(), x.to_owned()))
        .collect();
    keyboard_buttons.push(month_row);

    for week in calendar_vec.chunks(7) {
        let week_row = week
            .iter()
            .map(|week| InlineKeyboardButton::callback(week, week))
            .collect();

        keyboard_buttons.push(week_row);
    }

    Ok(InlineKeyboardMarkup::new(keyboard_buttons))
}

#[cfg(test)]
mod tests {

    use chrono::{Datelike, NaiveDate, Utc};
    use chrono_tz::Tz;

    use super::CalendarError;

    #[test]
    fn calendar() -> Result<(), CalendarError> {
        let now = Utc::now().with_timezone(&Tz::Singapore);
        let res = super::calendar(now.day(), now.month(), now.year())?;
        println!("{res:?}");
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
