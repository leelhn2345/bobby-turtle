use chrono::Local;

pub fn datetime_now() -> String {
    let now = Local::now();
    now.format("%v\n%r").to_string()
}
