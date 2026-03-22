use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::MaybeSet;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DefaultId(i64);

impl DefaultId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    #[cfg(feature = "server")]
    pub fn as_inner(self) -> i64 {
        self.0
    }
}

impl FromStr for DefaultId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl std::fmt::Display for DefaultId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Default {
    pub id: DefaultId,
    pub order: Option<i32>,
    pub regexp: Option<String>,
    pub name: Option<String>,
    pub action: String,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default {
    pub fn as_title(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.id.0.to_string())
    }

    #[cfg(feature = "server")]
    pub fn test_phone_number(&self, phone_number: &str) -> bool {
        if let Some(regexp) = &self.regexp {
            regex::Regex::new(regexp)
                .map(|re| re.is_match(phone_number))
                .unwrap_or(false)
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NewDefault {
    pub order: Option<i32>,
    pub regexp: Option<String>,
    pub name: Option<String>,
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChangeDefault {
    pub id: DefaultId,
    pub order: MaybeSet<Option<i32>>,
    pub regexp: MaybeSet<Option<String>>,
    pub name: MaybeSet<Option<String>>,
    pub action: MaybeSet<String>,
}

#[cfg(feature = "server")]
pub struct DefaultList(Vec<Default>);

#[cfg(feature = "server")]
impl DefaultList {
    pub fn new(defaults: Vec<Default>) -> Self {
        Self(defaults)
    }

    // pub fn iter(&self) -> Iter<'_, Default> {
    //     self.0.iter()
    // }

    pub fn search_phone_number(&self, phone_number: &str) -> Option<&Default> {
        self.0.iter().find(|d| d.test_phone_number(phone_number))
    }
}
