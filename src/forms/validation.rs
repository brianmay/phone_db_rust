use chrono::{DateTime, FixedOffset, TimeDelta, Utc};
use tap::Pipe;

use super::{FieldValue, errors::ValidationError};

pub fn validate_field_value<T: FieldValue<RawValue = String, DerefValue = str>>(
    str: &str,
) -> Result<T, ValidationError> {
    T::from_raw(str).map_err(|err| ValidationError(err.to_string()))
}

pub fn validate_phone_number(str: &str) -> Result<String, ValidationError> {
    validate_field_value(str)
}

pub fn validate_action(str: &str) -> Result<String, ValidationError> {
    match validate_field_value(str) {
        Ok(action) => {
            if action == "voicemail" || action == "allow" {
                Ok(action)
            } else {
                Err(ValidationError(
                    "Action must be 'voicemail' or 'allow'".to_string(),
                ))
            }
        }
        Err(err) => Err(err),
    }
}

pub fn validate_name(str: &str) -> Result<Option<String>, ValidationError> {
    validate_field_value(str)
}

pub fn validate_username(str: &str) -> Result<String, ValidationError> {
    validate_field_value(str)
}

pub fn validate_full_name(str: &str) -> Result<String, ValidationError> {
    validate_field_value(str)
}

pub fn validate_email(str: &str) -> Result<String, ValidationError> {
    let str = validate_field_value::<String>(str)?;
    if !str.contains('@') {
        return Err(ValidationError("Email should contain @".to_string()));
    }
    Ok(str)
}

pub fn validate_password(str: &str) -> Result<String, ValidationError> {
    validate_field_value(str)
}

pub fn validate_1st_password(str: &str) -> Result<String, ValidationError> {
    let str = validate_field_value::<String>(str)?;

    if str.is_empty() {
        return Err(ValidationError("Password cannot be empty".to_string()));
    }
    if str == "password" {
        return Err(ValidationError("Password cannot be 'password'".to_string()));
    }
    Ok(str)
}

pub fn validate_2nd_password(
    password_1: &Result<String, ValidationError>,
    password_2: &str,
) -> Result<String, ValidationError> {
    let password_2 = validate_field_value::<String>(password_2)?;
    let password_1 = password_1
        .as_ref()
        .map_err(|_err| ValidationError("Passwords do not match".to_string()))?;
    if *password_1 != password_2 {
        return Err(ValidationError("Passwords do not match".to_string()));
    }
    Ok(password_2)
}

pub fn validate_comments(str: &str) -> Result<Option<String>, ValidationError> {
    validate_field_value(str)
}

// pub fn validate_utc_date_time(str: &str) -> Result<chrono::DateTime<Utc>, ValidationError> {
//     validate_field_value(str)
// }

pub fn validate_fixed_offset_date_time(
    str: &str,
) -> Result<chrono::DateTime<FixedOffset>, ValidationError> {
    validate_field_value(str)
}

pub fn validate_maybe_date_time(str: &str) -> Result<Option<DateTime<Utc>>, ValidationError> {
    validate_field_value(str)
}

pub fn validate_duration(str: &str) -> Result<TimeDelta, ValidationError> {
    validate_field_value(str)
}

pub fn validate_in_range<T>(str: &str, min: T, max: T) -> Result<T, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_in_range_inner(str, min, max, true)
}

pub fn validate_in_range_exclusive<T>(str: &str, min: T, max: T) -> Result<T, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_in_range_inner(str, min, max, false)
}

fn check_range<T>(v: T, min: &T, max: &T, max_inclusive: bool) -> Result<T, ValidationError>
where
    T: PartialOrd + std::fmt::Display,
{
    let min_ok = *min <= v;
    let max_ok = if max_inclusive { v <= *max } else { v < *max };

    if min_ok && max_ok {
        Ok(v)
    } else {
        let range_msg = if max_inclusive {
            format!("between {} and {}", min, max)
        } else {
            format!("between {} and {} (exclusive)", min, max)
        };
        Err(ValidationError(format!("Value must be {}", range_msg)))
    }
}

fn validate_in_range_inner<T>(
    str: &str,
    min: T,
    max: T,
    max_inclusive: bool,
) -> Result<T, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_field_value::<T>(str)?.pipe(|v: T| check_range(v, &min, &max, max_inclusive))
}

pub fn validate_in_range_maybe<T>(str: &str, min: T, max: T) -> Result<Option<T>, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_in_range_maybe_inner(str, min, max, true)
}

pub fn validate_in_range_maybe_exclusive<T>(
    str: &str,
    min: T,
    max: T,
) -> Result<Option<T>, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_in_range_maybe_inner(str, min, max, false)
}

fn validate_in_range_maybe_inner<T>(
    str: &str,
    min: T,
    max: T,
    max_inclusive: bool,
) -> Result<Option<T>, ValidationError>
where
    T: FieldValue<RawValue = String, DerefValue = str> + PartialOrd + std::fmt::Display,
{
    validate_field_value::<Option<T>>(str)?
        .map(|v: T| check_range(v, &min, &max, max_inclusive))
        .transpose()
}
