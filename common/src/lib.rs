use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct PhoneCall {
    pub text: String,
    pub rating: u8,
}
