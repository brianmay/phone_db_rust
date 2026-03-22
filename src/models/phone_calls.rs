use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::contacts::ContactId;

#[cfg(feature = "server")]
use super::common::MaybeSet;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhoneCallId(i64);

impl PhoneCallId {
    #[cfg(feature = "server")]
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    #[cfg(feature = "server")]
    pub fn as_inner(self) -> i64 {
        self.0
    }
}

impl FromStr for PhoneCallId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl std::fmt::Display for PhoneCallId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct PhoneCall {
    pub id: PhoneCallId,
    pub action: String,
    pub contact_id: ContactId,
    pub destination_number: Option<String>,
    pub source_number: String,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NewPhoneCall {
    pub action: String,
    pub contact_id: ContactId,
    pub destination_number: Option<String>,
    pub source_number: String,
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChangePhoneCall {
    pub action: MaybeSet<String>,
    pub contact_id: MaybeSet<ContactId>,
    pub destination_number: MaybeSet<Option<String>>,
    pub source_number: MaybeSet<String>,
}
