use core::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::slice::Iter;

use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub sub: String,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "status")]
pub enum Response<T> {
    Success { data: T },
    Error { message: String },
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Default)]
pub enum Action {
    #[default]
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "voicemail")]
    VoiceMail,
}

impl Action {
    // for db conversions
    pub fn as_str(&self) -> &str {
        match self {
            Action::Allow => "allow",
            Action::VoiceMail => "voicemail",
        }
    }

    pub fn get_all_options_as_str() -> Vec<(&'static str, &'static str)> {
        vec![("Allow", "allow"), ("Voice mail", "voicemail")]
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Action::Allow => write!(f, "Allow"),
            Action::VoiceMail => write!(f, "Voice mail"),
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid action: {0}")]
pub struct InvalidActionError(String);

impl TryFrom<&str> for Action {
    type Error = InvalidActionError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "allow" => Ok(Action::Allow),
            "voicemail" => Ok(Action::VoiceMail),
            _ => Err(InvalidActionError(s.to_string())),
        }
    }
}

impl From<String> for Action {
    // for db conversions
    fn from(s: String) -> Self {
        match s.as_str() {
            "allow" => Action::Allow,
            "voicemail" => Action::VoiceMail,
            _ => Action::Allow,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PhoneCallDetails {
    pub id: i64,
    pub action: Action,
    pub contact_id: i64,
    pub contact_name: Option<String>,
    pub contact_phone_number: String,
    pub contact_action: Action,
    pub contact_comments: Option<String>,
    pub destination_number: Option<String>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub number_calls: Option<i64>,
}

impl PhoneCallDetails {
    pub fn get_key(&self) -> PhoneCallKey {
        PhoneCallKey {
            inserted_at: self.inserted_at,
            id: self.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PhoneCallKey {
    pub inserted_at: DateTime<Utc>,
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContactDetails {
    pub id: i64,
    pub phone_number: String,
    pub name: Option<String>,
    pub action: Action,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments: Option<String>,
    pub number_calls: Option<i64>,
}

impl ContactDetails {
    pub fn get_update_request(
        self,
        name: Option<String>,
        action: Action,
        comments: Option<String>,
    ) -> ContactUpdateRequest {
        ContactUpdateRequest {
            id: self.id,
            name,
            action,
            comments,
        }
    }

    pub fn get_key(&self) -> ContactKey {
        ContactKey {
            phone_number: self.phone_number.clone(),
            id: self.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContactKey {
    pub phone_number: String,
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ContactUpdateRequest {
    pub id: i64,
    pub name: Option<String>,
    pub action: Action,
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IncomingPhoneCallRequest {
    pub phone_number: String,
    pub destination_number: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IncomingPhoneCallResponse {
    pub name: Option<String>,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageRequest<T> {
    pub after_key: Option<T>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Page<T, K> {
    pub items: Vec<T>,
    pub next_key: Option<K>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Default {
    pub id: i64,
    pub order: i32,
    pub regexp: String,
    pub name: String,
    pub action: Action,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default {
    pub fn test_phone_number(&self, phone_number: &str) -> bool {
        regex::Regex::new(&self.regexp)
            .map(|re| re.is_match(phone_number))
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultUpdateRequest {
    pub id: i64,
    pub order: i32,
    pub regexp: String,
    pub name: String,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultAddRequest {
    pub order: i32,
    pub regexp: String,
    pub name: String,
    pub action: Action,
}

pub struct DefaultList(Vec<Default>);

impl DefaultList {
    pub fn new(defaults: Vec<Default>) -> Self {
        Self(defaults)
    }

    pub fn iter(&self) -> Iter<Default> {
        self.0.iter()
    }

    pub fn search_phone_number(&self, phone_number: &str) -> Option<&Default> {
        self.0.iter().find(|d| d.test_phone_number(phone_number))
    }
}
