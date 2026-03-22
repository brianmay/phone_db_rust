use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::MaybeSet;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContactId(i64);

impl ContactId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    #[cfg(feature = "server")]
    pub fn as_inner(self) -> i64 {
        self.0
    }
}

impl FromStr for ContactId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl std::fmt::Display for ContactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Contact {
    pub id: ContactId,
    pub phone_number: String,
    pub name: Option<String>,
    pub action: String,
    pub comments: Option<String>,
    pub phone_call_count: i64,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    pub fn as_title(&self) -> String {
        if let Some(name) = &self.name {
            format!("{} ({})", name, self.phone_number)
        } else {
            self.phone_number.clone()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NewContact {
    pub phone_number: String,
    pub name: Option<String>,
    pub action: String,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChangeContact {
    pub phone_number: MaybeSet<String>,
    pub name: MaybeSet<Option<String>>,
    pub action: MaybeSet<String>,
    pub comments: MaybeSet<Option<String>>,
}
