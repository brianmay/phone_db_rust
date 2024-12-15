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

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
pub enum Action {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PhoneCall {
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
}

impl PhoneCall {
    pub fn get_key(&self) -> PhoneCallKey {
        PhoneCallKey {
            inserted_at: self.inserted_at,
            id: self.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PhoneCallKey {
    pub inserted_at: DateTime<Utc>,
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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
}
