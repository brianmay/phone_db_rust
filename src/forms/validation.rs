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
