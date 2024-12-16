use chrono::{DateTime, Local, Utc};

#[must_use]
pub fn datetime_to_string(dt: DateTime<Utc>) -> String {
    let local = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M:%S %z").to_string()
}
