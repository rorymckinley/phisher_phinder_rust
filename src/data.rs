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
    pub from: Vec<EmailAddressData>,
    pub reply_to: Vec<EmailAddressData>,
    pub return_path: Vec<EmailAddressData>
}

impl SenderAddresses {
    pub fn to_email_address_data(address: String) -> EmailAddressData {
        EmailAddressData {address, domain: None, registrar: None}
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct EmailAddressData {
    pub address: String,
    pub domain: Option<Domain>,
    pub registrar: Option<Registrar>,
}

#[cfg(test)]
mod to_email_address_data_tests {
    use super::*;

    #[test]
    fn sets_domain_to_none_if_domain_not_open_email_provider() {
        let address = "scammer@fake.zzz";
        let expected = EmailAddressData {
            address: address.into(),
            domain: None,
            registrar: None,
        };

        assert_eq!(expected, EmailAddressData::from_email_address(address))
    }

    // #[test]
    // fn sets_domain_if_domain_open_email_provider() {
    //     let address = "dirtyevilscammer@gmail.com";
    //     let expected = EmailAddressData {
    //         address: address.into(),
    //         domain: Some(
    //             Domain {
    //             }
    //         )
    //     };
    //
    //     assert_eq!(expected, EmailAddressData::from_email_address(address))
    // }
}

impl EmailAddressData {
    pub fn from_email_address(address: &str) -> Self {
        Self {
            address: address.into(),
            domain: None,
            registrar: None,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Domain {
    pub abuse_email_address: Option<String>,
    pub category: DomainCategory,
    pub name: String,
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

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Registrar {
    pub abuse_email_address: Option<String>,
    pub name: Option<String>,
}
