use core::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

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
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for Action {
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
