use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct OutputData {
    pub parsed_mail: ParsedMail,
}

impl OutputData {
    pub fn new(
        subject: Option<String>,
        email_addresses: EmailAddresses,
        links: Vec<Link>,
    ) -> Self {
        Self {
            parsed_mail: ParsedMail {
                links,
                subject,
                email_addresses,
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParsedMail {
    pub email_addresses: EmailAddresses,
    pub links: Vec<Link>,
    pub subject: Option<String>,
}

#[cfg(test)]
mod link_tests {
    use super::*;

    #[test]
    fn new_other_domain() {
        let url = "https://foo.bar";

        let expected = Link {
            href: url.into(),
            category: LinkCategory::Other,
        };

        assert_eq!(expected, Link::new(url))
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Link {
    category: LinkCategory,
    pub href: String,
}

impl Link {
    pub fn new(href: &str) -> Self {
        Self { href: href.into(), category: LinkCategory::Other }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum LinkCategory {
    Other
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct EmailAddresses {
    pub from: Vec<EmailAddressData>,
    pub links: Vec<EmailAddressData>,
    pub reply_to: Vec<EmailAddressData>,
    pub return_path: Vec<EmailAddressData>,
}

impl EmailAddresses {
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
mod email_address_date_from_email_address {
    use super::*;

    #[test]
    fn sets_domain_for_other_domain() {
        let address = "scammer@fake.zzz";
        let expected = EmailAddressData {
            address: address.into(),
            domain: Some(
                Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "fake.zzz".into(),
                    registration_date: None,
                }
            ),
            registrar: None,
        };

        assert_eq!(expected, EmailAddressData::from_email_address(address))
    }

    #[test]
    fn sets_domain_if_domain_open_email_provider() {
        let address = "dirtyevilscammer@gmail.com";
        let expected = EmailAddressData {
            address: address.into(),
            domain: Some(
                Domain {
                    abuse_email_address: None,
                    category: DomainCategory::OpenEmailProvider,
                    name: "gmail.com".into(),
                    registration_date: None,
                }
            ),
            registrar: None,
        };

        assert_eq!(expected, EmailAddressData::from_email_address(address))
    }
}

impl EmailAddressData {
    pub fn from_email_address(address: &str) -> Self {

        Self {
            address: address.into(),
            domain: Domain::from_email_address(address),
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

#[cfg(test)]
mod domain_from_email_address_tests {
    use super::*;

    #[test]
    fn other_domain() {
        let expected = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "test.xxx".into(),
            registration_date: None,
        };

        assert_eq!(Some(expected), Domain::from_email_address("foo@test.xxx"))
    }

    #[test]
    fn email_provider_domain() {
        let expected = Domain {
            abuse_email_address: None,
            category: DomainCategory::OpenEmailProvider,
            name: "outlook.com".into(),
            registration_date: None,
        };

        assert_eq!(Some(expected), Domain::from_email_address("foo@outlook.com"))
    }

    #[test]
    fn address_cannot_be_parsed() {
        assert_eq!(None, Domain::from_email_address("foo"))
    }
}

impl Domain {
    pub fn from_email_address(address: &str) -> Option<Self> {
        if let Some((_local_part, domain)) = address.split_once('@') {
            let open_email_providers = &[
                "aol.com",
                "gmail.com",
                "googlemail.com",
                "hotmail.com",
                "outlook.com",
                "yahoo.com",
                "163.com"
            ];

            let category = if open_email_providers.contains(&domain) {
                DomainCategory::OpenEmailProvider
            } else {
                DomainCategory::Other
            };

            Some(
                Self {
                    abuse_email_address: None,
                    category,
                    name: domain.into(),
                    registration_date: None,
                }
            )
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainCategory {
    OpenEmailProvider,
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
