use chrono::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use url::Url;

use crate::authentication_results::AuthenticationResults;
use crate::message_source::MessageSource;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct OutputData {
    pub parsed_mail: ParsedMail,
    pub message_source: MessageSource,
    pub reportable_entities: Option<ReportableEntities>,
    pub run_id: Option<u32>,
}

impl OutputData {
    pub fn new(parsed_mail: ParsedMail, message_source: MessageSource) -> Self {
        Self {
            parsed_mail,
            message_source,
            reportable_entities: None,
            run_id: None,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParsedMail {
    pub authentication_results: Option<AuthenticationResults>,
    pub delivery_nodes: Vec<DeliveryNode>,
    pub email_addresses: EmailAddresses,
    pub fulfillment_nodes: Vec<FulfillmentNode>,
    pub subject: Option<String>,
}

impl ParsedMail {
    pub fn new(
        authentication_results: Option<AuthenticationResults>,
        delivery_nodes: Vec<DeliveryNode>,
        email_addresses: EmailAddresses,
        fulfillment_nodes: Vec<FulfillmentNode>,
        subject: Option<String>,
    ) -> Self {
        Self {
            authentication_results,
            delivery_nodes,
            email_addresses,
            fulfillment_nodes,
            subject,
        }
    }
}

#[cfg(test)]
mod delivery_node_tests {
    use super::*;

    #[test]
    fn builds_a_delivery_node_from_header_value() {
        let mut trusted_node = unobserved_trusted_node("b.baz.com");
        let value = header_value(
            ("a.bar.com", "b.bar.com.", "10.10.10.12"),
            "a.baz.com",
            "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)",
        );

        let expected = DeliveryNode {
            advertised_sender: host_node_option("a.bar.com", None),
            observed_sender: host_node_option("b.bar.com", Some("10.10.10.12")),
            position: 10,
            recipient: recipient_option(),
            time: date_option(),
            trusted: false,
        };

        assert_eq!(
            expected,
            DeliveryNode::from_header_value(&value, 10, &mut trusted_node)
        )
    }

    #[test]
    fn indicates_trusted_if_matches_trusted_node() {
        let mut trusted_node = unobserved_trusted_node("a.baz.com");
        let value = header_value(
            ("a.bar.com", "b.bar.com.", "10.10.10.12"),
            "a.baz.com",
            "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)",
        );

        let node = DeliveryNode::from_header_value(&value, 10, &mut trusted_node);

        assert!(node.trusted);
    }

    #[test]
    fn updates_trusted_node_if_matches_and_node_is_unassigned() {
        let mut trusted_node = unobserved_trusted_node("a.baz.com");
        let value = header_value(
            ("a.bar.com", "b.bar.com.", "10.10.10.12"),
            "a.baz.com",
            "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)",
        );

        DeliveryNode::from_header_value(&value, 10, &mut trusted_node);

        assert!(trusted_node.observed);
    }

    fn header_value(from_parts: (&str, &str, &str), by_host: &str, date: &str) -> String {
        let (advertised_host, actual_host, ip) = from_parts;

        let from = format!("{advertised_host} ({actual_host} [{ip}])");
        let by = format!("{by_host} with ESMTP id jg8-2002");
        let f_o_r = "<victim@gmail.com>";

        format!("from {from}\r\n        by {by}\r\n        for {f_o_r};\r\n        {date}")
    }

    fn host_node_option(host: &str, ip_address: Option<&str>) -> Option<HostNode> {
        Some(HostNode::new(Some(host), ip_address).unwrap())
    }

    fn recipient_option() -> Option<String> {
        Some("a.baz.com".into())
    }

    fn date_option() -> Option<DateTime<Utc>> {
        Some(Utc.with_ymd_and_hms(2022, 9, 6, 23, 17, 22).unwrap())
    }

    fn unobserved_trusted_node(recipient: &str) -> TrustedRecipientDeliveryNode {
        TrustedRecipientDeliveryNode {
            recipient: String::from(recipient),
            observed: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DeliveryNode {
    pub advertised_sender: Option<HostNode>,
    pub observed_sender: Option<HostNode>,
    pub position: usize,
    pub recipient: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub trusted: bool,
}

impl DeliveryNode {
    pub fn from_header_value(
        header_value: &str,
        position: usize,
        trusted_node: &mut TrustedRecipientDeliveryNode,
    ) -> Self {
        let recipient = extract_recipient(header_value);

        let trusted = trusted_node.check_if_trusted(recipient.as_deref());

        Self {
            advertised_sender: extract_advertised_sender(header_value),
            observed_sender: extract_observed_sender(header_value),
            position,
            recipient,
            time: extract_time_from_header(header_value),
            trusted,
        }
    }
}

#[cfg(test)]
mod extract_advertised_sender_tests {
    use super::*;

    #[test]
    fn returns_the_advertised_sender() {
        let expected = Some(HostNode::new(Some("a.bar.com"), None).unwrap());

        let header = header_value("a.bar.com", "b.bar.com.", "10.10.10.10");

        assert_eq!(expected, extract_advertised_sender(&header));
    }

    #[test]
    fn returns_none_if_no_from() {
        let header = no_from_header_value();

        assert_eq!(None, extract_advertised_sender(&header));
    }

    #[test]
    fn returns_none_if_empty_string() {
        assert_eq!(None, extract_advertised_sender(""));
    }

    // TODO Generalise this for use by all tests
    fn header_value(advertised_host: &str, observed_host: &str, ip: &str) -> String {
        let from = format!("{advertised_host} ({observed_host} [{ip}])");

        let rest_of_header = no_from_header_value();

        format!("from {from}\r\n        {rest_of_header}")
    }

    fn no_from_header_value() -> String {
        let by = "does.not.matter with ESMTP id jg8-2002";
        let f_o_r = "<victim@gmail.com>";
        let date = "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        format!("by {by}\r\n        for {f_o_r};\r\n        {date}")
    }
}

fn extract_advertised_sender(header_value: &str) -> Option<HostNode> {
    let regex = Regex::new(r"from\s(\S+)\s\(").unwrap();

    regex.captures(header_value).map(|captures| {
        HostNode::new(Some(&captures[1]), None).expect("Creating a HostNode for advertised sender")
    })
}

#[cfg(test)]
mod extract_observed_sender_tests {
    use super::*;

    #[test]
    fn returns_host_node_with_host_and_ip() {
        let header = header_value("a.bar.com", Some("b.bar.com."), Some("[10.10.10.10]"));

        let expected = Some(HostNode::new(Some("b.bar.com"), Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_host_and_ip_host_has_no_trailing_period() {
        let header = header_value("a.bar.com", Some("b.bar.com"), Some("[10.10.10.10]"));

        let expected = Some(HostNode::new(Some("b.bar.com"), Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_host_and_ip_ip_not_in_squares() {
        let header = header_value("a.bar.com", Some("b.bar.com."), Some("10.10.10.10"));

        let expected = Some(HostNode::new(Some("b.bar.com"), Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_ip_no_observed_host() {
        let header = header_value("a.bar.com", None, Some("[10.10.10.10]"));

        let expected = Some(HostNode::new(None, Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_ip_no_host_ip_not_in_squares() {
        let header = header_value("a.bar.com", None, Some("10.10.10.10"));

        let expected = Some(HostNode::new(None, Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_host_no_ip() {
        let header = header_value("a.bar.com", Some("b.bar.com."), None);

        let expected = Some(HostNode::new(Some("b.bar.com"), None).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_host_node_with_ip_no_host_but_ehlo() {
        let header = header_with_ehlo("10.10.10.10");
        println!("{header}");

        let expected = Some(HostNode::new(None, Some("10.10.10.10")).unwrap());

        assert_eq!(expected, extract_observed_sender(&header));
    }

    #[test]
    fn returns_none_if_no_observed_sender() {
        let header = no_observed_sender();

        assert_eq!(None, extract_observed_sender(&header));
    }

    #[test]
    fn returns_none_if_no_from_header() {
        let header = no_from_header_value();

        assert_eq!(None, extract_observed_sender(&header));
    }

    #[test]
    fn returns_none_if_empty_header() {
        assert_eq!(None, extract_observed_sender(""))
    }

    fn header_value(
        advertised_host: &str,
        observed_host_opt: Option<&str>,
        ip_opt: Option<&str>,
    ) -> String {
        let observed_host_padded = if let Some(observed_host) = observed_host_opt {
            format!("{observed_host} ")
        } else {
            String::from("")
        };

        let ip = ip_opt.unwrap_or("");

        let from = format!("{advertised_host} ({observed_host_padded}{ip})");

        let rest_of_header = no_from_header_value();

        format!("from {from}\r\n        {rest_of_header}")
    }

    fn header_with_ehlo(ip: &str) -> String {
        let from = format!("10.217.130.145 (EHLO foo.bar.baz) ({ip})");

        let rest_of_header = no_from_header_value();

        format!("from {from}\r\n        {rest_of_header}")
    }

    fn no_from_header_value() -> String {
        let by = "does.not.matter with ESMTP id jg8-2002";
        let f_o_r = "<victim@gmail.com>";
        let date = "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        format!("by {by}\r\n        for {f_o_r};\r\n        {date}")
    }

    fn no_observed_sender() -> String {
        let from = "does.not.matter ()";

        let rest_of_header = no_from_header_value();

        format!("from {from}\r\n        {rest_of_header}")
    }
}

fn extract_observed_sender(header_value: &str) -> Option<HostNode> {
    let pattern = format!(
        r"{}\({}{}\)",
        observed_sender_ignore_snippet(),
        observed_sender_observed_host_snippet(),
        observed_sender_observed_ip_snippet()
    );
    let regex = Regex::new(&pattern).unwrap();

    if let Some(captures) = regex.captures(header_value) {
        HostNode::new(
            captures.name("host").map(|m| m.as_str()),
            captures.name("ip").map(|m| m.as_str()),
        )
        .ok()
    } else {
        None
    }
}

fn observed_sender_ignore_snippet() -> String {
    r"from\s\S+\s(\(EHLO[^\)]+\)\s)?".into()
}

fn observed_sender_observed_host_snippet() -> String {
    r"((?P<host>\S+?)\.?\s)?".into()
}

fn observed_sender_observed_ip_snippet() -> String {
    r"(\[?(?P<ip>[[A-Za-z0-9.:]+]+)\]?)?".into()
}

#[cfg(test)]
mod extract_recipient_tests {
    use super::*;

    #[test]
    fn returns_name_of_recipient() {
        let header = header_value("a.baz.com");

        let expected = Some("a.baz.com".into());

        assert_eq!(expected, extract_recipient(&header));
    }

    #[test]
    fn returns_name_of_recipient_if_no_from_section() {
        let header = no_from_header_value("a.baz.com");

        let expected = Some("a.baz.com".into());

        assert_eq!(expected, extract_recipient(&header));
    }

    #[test]
    fn returns_none_if_no_by_section() {
        let header = no_by_header_value();

        assert_eq!(None, extract_recipient(&header))
    }

    #[test]
    fn returns_none_if_empty_string() {
        assert_eq!(None, extract_recipient(""))
    }

    fn header_value(recipient: &str) -> String {
        let from = String::from("does.not.matter (does.not.matter [10.10.10.10])");
        let by = format!("{recipient} (Postfix) with ESMTP id jg8-2002");
        let f_o_r = "<victim@gmail.com>";
        let date = "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        format!("from {from}\r\n        by {by}\r\n        for {f_o_r};\r\n        {date}")
    }

    fn no_from_header_value(recipient: &str) -> String {
        let by = format!("{recipient} does.not.matter with ESMTP id jg8-2002");
        let f_o_r = "<victim@gmail.com>";
        let date = "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        format!("by {by}\r\n        for {f_o_r};\r\n        {date}")
    }

    fn no_by_header_value() -> String {
        let from = String::from("does.not.matter (does.not.matter [10.10.10.10])");
        let f_o_r = "<victim@gmail.com>";
        let date = "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        format!("from {from}\r\n        for {f_o_r};\r\n        {date}")
    }
}

fn extract_recipient(header_value: &str) -> Option<String> {
    let regex = Regex::new(r"by\s(?P<recipient>\S+)\s").unwrap();

    regex
        .captures(header_value)
        .map(|captures| captures["recipient"].into())
}

#[cfg(test)]
mod extract_time_from_header_tests {
    use super::*;

    #[test]
    fn parses_rfc_2822_time_component() {
        let header_value = "does not matter;\r\n        Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        let expected = Utc.with_ymd_and_hms(2022, 9, 6, 23, 17, 22).unwrap();

        assert_eq!(Some(expected), extract_time_from_header(header_value));
    }

    #[test]
    fn returns_none_when_empty_string() {
        assert_eq!(None, extract_time_from_header(""))
    }

    #[test]
    fn returns_none_when_not_rfc_compliant_no_semicolon() {
        let header_value = "does not matter\r\n        Tue, 06 Sep 2022 16:17:22 -0700 (PDT)";

        assert_eq!(None, extract_time_from_header(header_value));
    }

    #[test]
    fn returns_none_when_date_is_not_parseable() {
        let header_value = "dnm;\r\n        2023-03-02 17:20:58.194078568 +0000 UTC m=+755221.897";

        assert_eq!(None, extract_time_from_header(header_value));
    }
}

fn extract_time_from_header(header_value: &str) -> Option<DateTime<Utc>> {
    let header_parts = header_value.split(';').collect::<Vec<&str>>();

    if let &[_, date_part] = header_parts.as_slice() {
        match DateTime::parse_from_rfc2822(date_part.trim()) {
            Ok(date) => Some(date.into()),
            Err(_) => None,
        }
    } else {
        None
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
            },
        };

        assert_eq!(expected, FulfillmentNode::new(url));
    }

    #[test]
    fn visible_url_test() {
        let f_node = FulfillmentNode {
            hidden: Some(Node::new("https://foo.bar")),
            visible: Node::new("https://foo.baz"),
        };

        assert_eq!("https://foo.baz", f_node.visible_url());
    }

    #[test]
    fn hidden_url_with_hidden_domain_test() {
        let mut f_node = FulfillmentNode::new("https://foo.bar");
        f_node.set_hidden("https://redirect.bar");

        assert_eq!(
            Some(String::from("https://redirect.bar")),
            f_node.hidden_url()
        );
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FulfillmentNode {
    pub hidden: Option<Node>,
    pub visible: Node,
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
mod host_node_tests {
    use super::*;

    #[test]
    fn new_with_just_host() {
        let host = "foo.bar";
        let ip_address = None;

        let expected = HostNode {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "foo.bar".into(),
                registration_date: None,
            }),
            host: Some(host.into()),
            infrastructure_provider: None,
            ip_address: None,
            registrar: None,
        };

        assert_eq!(expected, HostNode::new(Some(host), ip_address).unwrap())
    }

    #[test]
    fn new_with_host_and_ip_address() {
        let host = "foo.bar";
        let ip_address = Some("10.10.10.10");

        let expected = HostNode {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "foo.bar".into(),
                registration_date: None,
            }),
            host: Some(host.into()),
            infrastructure_provider: None,
            ip_address: Some("10.10.10.10".into()),
            registrar: None,
        };

        assert_eq!(expected, HostNode::new(Some(host), ip_address).unwrap())
    }

    #[test]
    fn new_with_just_ip() {
        let ip_address = Some("10.10.10.10");

        let expected = HostNode {
            domain: None,
            host: None,
            infrastructure_provider: None,
            ip_address: Some("10.10.10.10".into()),
            registrar: None,
        };

        assert_eq!(expected, HostNode::new(None, ip_address).unwrap())
    }

    #[test]
    fn new_without_host_and_ip() {
        match HostNode::new(None, None) {
            Err(_) => (),
            Ok(_) => panic!("Returned OK"),
        }
    }
}

#[derive(Debug, Error)]
pub enum HostNodeError {
    #[error("error instantiating HostNode")]
    InstantiationError,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HostNode {
    pub domain: Option<Domain>,
    pub host: Option<String>,
    pub infrastructure_provider: Option<InfrastructureProvider>,
    pub ip_address: Option<String>,
    pub registrar: Option<Registrar>,
}

impl HostNode {
    pub fn new(host: Option<&str>, ip_address: Option<&str>) -> Result<Self, HostNodeError> {
        if let (None, None) = (host, ip_address) {
            return Err(HostNodeError::InstantiationError);
        }

        let domain = match host {
            Some(h) => Domain::from_host(h),
            None => None,
        };

        Ok(Self {
            domain,
            host: host.map(|h| h.into()),
            infrastructure_provider: None,
            ip_address: ip_address.map(|ip_a| ip_a.into()),
            registrar: None,
        })
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Node {
    pub domain: Option<Domain>,
    pub registrar: Option<Registrar>,
    pub url: String,
}

impl Node {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.into(),
            domain: Domain::from_url(url),
            registrar: None,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum LinkCategory {
    Other,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EmailAddresses {
    pub from: Vec<EmailAddressData>,
    pub links: Vec<EmailAddressData>,
    pub reply_to: Vec<EmailAddressData>,
    pub return_path: Vec<EmailAddressData>,
}

impl EmailAddresses {
    pub fn to_email_address_data(address: String) -> EmailAddressData {
        EmailAddressData {
            address,
            domain: None,
            registrar: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EmailAddressData {
    pub address: String,
    pub domain: Option<Domain>,
    pub registrar: Option<Registrar>,
}

#[cfg(test)]
mod email_address_data_from_email_address {
    use super::*;

    #[test]
    fn sets_domain_for_other_domain() {
        let address = "scammer@fake.zzz";
        let expected = EmailAddressData {
            address: address.into(),
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "fake.zzz".into(),
                registration_date: None,
            }),
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
            abuse_email_address: Some("abuse@outlook.com".into()),
            category: DomainCategory::OpenEmailProvider,
            name: "outlook.com".into(),
            registration_date: None,
        };

        assert_eq!(
            Some(expected),
            Domain::from_email_address("foo@outlook.com")
        )
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
    fn creates_domain_instance_for_url_shortener() {
        let url = "https://tinyurl.com/42jxt5p6";

        let expected = Domain {
            abuse_email_address: Some("abuse@tinyurl.com".into()),
            category: DomainCategory::UrlShortener,
            name: "tinyurl.com".into(),
            registration_date: None,
        };

        assert_eq!(Some(expected), Domain::from_url(url));
    }

    #[test]
    fn returns_none_if_domain_cannot_be_parsed() {
        let url = "foo.baz";

        assert_eq!(None, Domain::from_url(url));
    }

    #[test]
    fn returns_none_if_no_host_name() {
        let url = "unix:/run/foo.socket";

        assert_eq!(None, Domain::from_url(url));
    }
}

#[cfg(test)]
mod from_host_tests {
    use super::*;

    #[test]
    fn instantiates_a_domain() {
        let host = "foo.baz";

        let expected = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "foo.baz".into(),
            registration_date: None,
        };

        assert_eq!(Some(expected), Domain::from_host(host));
    }

    #[test]
    fn does_not_instantiate_if_host_string_is_empty() {
        assert_eq!(None, Domain::from_host(""));
    }
}

impl Domain {
    pub fn from_email_address(address: &str) -> Option<Self> {
        if let Some((_local_part, domain)) = address.split_once('@') {
            let providers = EmailProviders::new();

            if providers.is_member(domain) {
                Some(Self::initialise_email_provider_domain(domain))
            } else {
                Some(Self::initialise_other_domain(domain))
            }
        } else {
            None
        }
    }

    pub fn from_url(url_str: &str) -> Option<Self> {
        if let Ok(url) = Url::parse(url_str) {
            match url.host_str() {
                Some(name) => {
                    let providers = ShortenedUrlProviders::new();

                    if providers.is_member(name) {
                        Some(Self::initialise_url_shortener_domain(name))
                    } else {
                        Some(Self::initialise_other_domain(name))
                    }
                }
                None => None,
            }
        } else {
            None
        }
    }

    pub fn from_host(host: &str) -> Option<Self> {
        if host.is_empty() {
            None
        } else {
            Some(Self::initialise_other_domain(host))
        }
    }

    fn initialise_email_provider_domain(domain: &str) -> Self {
        let providers = EmailProviders::new();

        Self {
            abuse_email_address: providers.abuse_address(domain).map(|addr| addr.into()),
            category: DomainCategory::OpenEmailProvider,
            name: domain.into(),
            registration_date: None,
        }
    }

    fn initialise_other_domain(domain: &str) -> Self {
        Self {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: domain.into(),
            registration_date: None,
        }
    }

    fn initialise_url_shortener_domain(domain: &str) -> Self {
        let providers = ShortenedUrlProviders::new();

        Self {
            abuse_email_address: providers.abuse_address(domain).map(|addr| addr.into()),
            category: DomainCategory::UrlShortener,
            name: domain.into(),
            registration_date: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainCategory {
    OpenEmailProvider,
    Other,
    UrlShortener,
}

#[cfg(test)]
mod shortened_url_providers {
    use super::*;

    #[test]
    fn indicates_if_domain_is_member() {
        let sup = ShortenedUrlProviders::new();

        assert!(!sup.is_member("foo.bar"));
        assert!(sup.is_member("rb.gy"));
    }

    #[test]
    fn returns_the_abuse_address_if_domain_is_member() {
        let sup = ShortenedUrlProviders::new();

        assert_eq!(Some("support@rebrandly.com"), sup.abuse_address("rb.gy"));
    }

    #[test]
    fn returns_none_if_domiain_is_not_member() {
        let sup = ShortenedUrlProviders::new();

        assert_eq!(None, sup.abuse_address("foo.bar"));
    }
}

struct ShortenedUrlProviders {
    providers: HashMap<String, String>,
}

impl ShortenedUrlProviders {
    fn new() -> Self {
        Self {
            providers: HashMap::from([
                ("bit.ly".into(), "abuse@bitly.com".into()),
                ("ow.ly".into(), "abuse@hootsuite.com".into()),
                ("rb.gy".into(), "support@rebrandly.com".into()),
                ("shorte.st".into(), "tcoabuse@twitter.com".into()),
                ("t.ly".into(), "support@t.ly".into()),
                ("t.co".into(), "tcoabuse@twitter.com".into()),
                ("tiny.cc".into(), "abuse@tiny.cc".into()),
                ("tinyurl.com".into(), "abuse@tinyurl.com".into()),
            ]),
        }
    }

    pub fn is_member(&self, domain_name: &str) -> bool {
        self.providers.get_key_value(domain_name).is_some()
    }

    pub fn abuse_address(&self, domain_name: &str) -> Option<&str> {
        self.providers
            .get_key_value(domain_name)
            .map(|(_, val)| &val[..])
    }
}

#[cfg(test)]
mod email_providers_tests {
    use super::*;

    #[test]
    fn indicates_if_domain_is_a_number() {
        let providers = EmailProviders::new();

        assert!(!providers.is_member("test.xxx"));
        assert!(providers.is_member("outlook.com"))
    }

    #[test]
    fn returns_abuse_address_if_domain_is_a_number() {
        let providers = EmailProviders::new();

        assert_eq!(
            Some("abuse@gmail.com"),
            providers.abuse_address("googlemail.com")
        );
    }

    #[test]
    fn returns_none_if_domain_is_not_a_member() {
        let providers = EmailProviders::new();

        assert!(providers.abuse_address("test.xxx").is_none())
    }
}

struct EmailProviders {
    providers: HashMap<String, String>,
}

impl EmailProviders {
    fn new() -> Self {
        Self {
            providers: HashMap::from([
                ("163.com".into(), "abuse@163.com".into()),
                ("aol.com".into(), "abuse@aol.com".into()),
                ("gmail.com".into(), "abuse@gmail.com".into()),
                ("googlemail.com".into(), "abuse@gmail.com".into()),
                ("hotmail.com".into(), "abuse@hotmail.com".into()),
                ("is.gd".into(), "abuse@is.gd".into()),
                ("outlook.com".into(), "abuse@outlook.com".into()),
                ("yahoo.com".into(), "abuse@yahoo.com".into()),
            ]),
        }
    }

    pub fn is_member(&self, domain_name: &str) -> bool {
        self.providers.get_key_value(domain_name).is_some()
    }

    pub fn abuse_address(&self, domain_name: &str) -> Option<&str> {
        self.providers
            .get_key_value(domain_name)
            .map(|(_, val)| &val[..])
    }
}

impl fmt::Display for DomainCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Registrar {
    pub abuse_email_address: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InfrastructureProvider {
    pub abuse_email_address: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct TrustedRecipientDeliveryNode {
    pub recipient: String,
    pub observed: bool,
}

#[cfg(test)]
mod trusted_recipient_delivery_node_new_tests {
    use super::*;

    #[test]
    fn can_instantiate_itself_from_a_name() {
        let expected = TrustedRecipientDeliveryNode {
            recipient: String::from("foo"),
            observed: false,
        };

        assert_eq!(expected, TrustedRecipientDeliveryNode::new("foo"));
    }
}

#[cfg(test)]
mod trusted_recipient_delivery_node_check_if_trusted_tests {
    use super::*;

    #[test]
    fn first_observation_returns_true_if_node_recipient_matches_recipient() {
        let mut node = trusted_node(false);

        assert!(node.check_if_trusted(Some("trusted_recipient")));
    }

    #[test]
    fn first_observation_indicates_node_has_been_observed() {
        let mut node = trusted_node(false);

        node.check_if_trusted(Some("trusted_recipient"));

        assert!(node.observed);
    }

    #[test]
    fn first_observation_returns_false_if_node_recipient_does_not_match_candidate() {
        let mut node = trusted_node(false);

        assert!(!node.check_if_trusted(Some("not_trusted_recipient")));
    }

    #[test]
    fn first_observation_returns_false_if_no_candidate() {
        let mut node = trusted_node(false);

        assert!(!node.check_if_trusted(None));
    }

    #[test]
    fn already_observed_always_returns_false() {
        let mut node = trusted_node(true);

        assert!(!node.check_if_trusted(Some("trusted_recipient")));
    }

    fn trusted_node(observed: bool) -> TrustedRecipientDeliveryNode {
        TrustedRecipientDeliveryNode {
            recipient: "trusted_recipient".into(),
            observed,
        }
    }
}

impl TrustedRecipientDeliveryNode {
    pub fn new(name: &str) -> Self {
        Self {
            recipient: String::from(name),
            observed: false,
        }
    }

    pub fn check_if_trusted(&mut self, candidate_option: Option<&str>) -> bool {
        if self.observed {
            return false;
        }

        match candidate_option {
            Some(candidate) => {
                if self.recipient == candidate {
                    self.observed = true;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ReportableEntities {
    pub delivery_nodes: Vec<DeliveryNode>,
    pub email_addresses: EmailAddresses,
    pub fulfillment_nodes: Vec<FulfillmentNode>,
}
