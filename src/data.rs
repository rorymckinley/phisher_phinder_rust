use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct OutputData {
    pub parsed_mail: ParsedMail,
    pub raw_mail: String,
}

impl OutputData {
    pub fn new(
        parsed_mail: ParsedMail,
        raw_mail: &str,
    ) -> Self {
        Self {
            parsed_mail,
            raw_mail: raw_mail.into()
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParsedMail {
    pub email_addresses: EmailAddresses,
    pub fulfillment_nodes: Vec<FulfillmentNode>,
    pub subject: Option<String>,
}

impl ParsedMail {
    pub fn new(
        email_addresses: EmailAddresses,
        fulfillment_nodes: Vec<FulfillmentNode>,
        subject: Option<String>
    ) -> Self {
        Self { email_addresses, fulfillment_nodes, subject }
    }
}

#[cfg(test)]
mod fulfillment_node_tests {
    use super::*;

    #[test]
    fn new_test() {
        let url = "https://foo.bar";

        let expected = FulfillmentNode {
            hidden: None,
            visible: Node {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "foo.bar".into(),
                    registration_date: None,
                }),
                registrar: None,
                url: url.into(),
            }
        };

        assert_eq!(expected, FulfillmentNode::new(url));
    }

    #[test]
    fn visible_url_test() {
        let f_node = FulfillmentNode {
            hidden: Some(Node::new("https://foo.bar")),
            visible: Node::new("https://foo.baz")
        };

        assert_eq!("https://foo.baz", f_node.visible_url());
    }

    #[test]
    fn hidden_url_with_hidden_domain_test() {
        let mut f_node = FulfillmentNode::new("https://foo.bar");
        f_node.set_hidden("https://redirect.bar");

        assert_eq!(Some(String::from("https://redirect.bar")), f_node.hidden_url());
    }

    #[test]
    fn hidden_url_with_no_hidden_domain_test() {
        let f_node = FulfillmentNode::new("https://foo.bar");

        assert_eq!(None, f_node.hidden_url());
    }

    #[test]
    fn set_hidden_test() {
        let mut f_node = FulfillmentNode::new("https://foo.baz");

        f_node.set_hidden("https://foo.bar");

        assert_eq!(Node::new("https://foo.baz"), f_node.visible);
        assert_eq!(Some(Node::new("https://foo.bar")), f_node.hidden);
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FulfillmentNode {
    pub hidden: Option<Node>,
    pub visible: Node
}

impl FulfillmentNode {
    pub fn new(visible_url: &str) -> Self {
        Self {
            hidden: None,
            visible: Node::new(visible_url),
        }
    }

    // TODO Return string rather to align with .hidden_url()?
    pub fn visible_url(&self) -> &str {
        &self.visible.url
    }

    pub fn hidden_url(&self) -> Option<String> {
        self.hidden.as_ref().map(|node| node.url.clone())
    }

    pub fn set_hidden(&mut self, url: &str) {
        self.hidden = Some(Node::new(url));
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;

    #[test]
    fn test_new() {
        let url = "https://foo.bar";

        let expected = Node {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "foo.bar".into(),
                registration_date: None,
            }),
            registrar: None,
            url: url.into(),
        };

        assert_eq!(expected, Node::new(url))
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Node {
    pub domain: Option<Domain>,
    pub registrar: Option<Registrar>,
    pub url: String,
}

impl Node {
    pub fn new(url: &str) -> Self {
        Self { url: url.into(), domain: Domain::from_url(url), registrar: None }
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

#[cfg(test)]
mod domain_from_url_tests {
    use super::*;

    #[test]
    fn creates_domain_instance() {
        let url = "https://foo.baz";

        let expected = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "foo.baz".into(),
            registration_date: None,
        };

        assert_eq!(Some(expected), Domain::from_url(url));
    }

    #[test]
    fn retuns_none_if_domain_cannot_be_parsed() {
        let url = "foo.baz";

        assert_eq!(None, Domain::from_url(url));
    }

    #[test]
    fn returns_none_if_no_host_name() {
        let url = "unix:/run/foo.socket";

        assert_eq!(None, Domain::from_url(url));
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

    pub fn from_url(url_str: &str) -> Option<Self> {
        if let Ok(url) = Url::parse(url_str) {
            url
                .host_str()
                .map(|name| {
                Self {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: name.into(),
                    registration_date: None,
                }
            })
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
