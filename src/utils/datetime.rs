use chrono::Local;

pub fn datetime_now() -> String {
    let now = Local::now();
    now.format("%v\n%r - GMT+8").to_string()
}
