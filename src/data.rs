use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct OutputData {
    pub parsed_mail: ParsedMail,
}

impl OutputData {
    pub fn new(subject: Option<String>, sender_addresses: SenderAddresses) -> Self {
        Self {
            parsed_mail: ParsedMail {
                subject,
                sender_addresses
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParsedMail {
    pub sender_addresses: SenderAddresses,
    pub subject: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SenderAddresses {
    pub from: Option<EmailAddressData>,
    pub reply_to: Option<EmailAddressData>,
    pub return_path: Option<EmailAddressData>
}

impl SenderAddresses {
    pub fn to_email_address_data(address: String) -> EmailAddressData {
        EmailAddressData {address, domain: None}
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct EmailAddressData {
    pub address: String,
    pub domain: Option<Domain>,
}

impl EmailAddressData {
    pub fn from_email_address(address: String) -> Self {
        Self {
            address,
            domain: None
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Domain {
    pub abuse_email_address: Option<String>,
    pub category: DomainCategory,
    pub name: String,
    pub registrar: Option<String>,
    pub registration_date: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainCategory {
    Other,
}

impl fmt::Display for DomainCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) ->fmt::Result {
        write!(f, "{:?}", self)
    }
}
