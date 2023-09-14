use chrono::Utc;
use chrono_tz::Singapore;

pub fn datetime_now() -> String {
    let now = Utc::now().with_timezone(&Singapore);
    now.format("%v\n%r - GMT+8").to_string()
}
