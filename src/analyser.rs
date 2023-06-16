use crate::analysable_message::AnalysableMessage;
use crate::authentication_results::AuthenticationResults;
use crate::data::{
    DeliveryNode,
    EmailAddressData,
    EmailAddresses,
    FulfillmentNode,
    TrustedRecipientDeliveryNode,
};
use regex::Regex;

pub struct Analyser<'a, T> {
    parsed_mail: &'a T
}

#[cfg(test)]
mod email_addresses_tests {
    use super::*;
    use crate::data::{Domain, DomainCategory};

    #[test]
    fn sender_email_addresses() {
        let parsed = parsed_mail(vec![
            "mailto:link2@test.com;link1@test.com",
        ]);
        let analyser = Analyser::new(&parsed);

        let expected_result = EmailAddresses {
            from: vec![convert_email_addresses("from@test.com")],
            reply_to: vec![convert_email_addresses("reply@test.com")],
            return_path: vec![convert_email_addresses("return@test.com")],
            links: vec![
                convert_email_addresses("link1@test.com"),
                convert_email_addresses("link2@test.com"),
            ],
        };

        assert_eq!(expected_result, analyser.sender_email_addresses())
    }

    #[test]
    fn sender_email_addresses_multiple_links() {
        let parsed = parsed_mail(vec![
            "mailto:link2@test.com;link1@test.com",
            "mailto:link3@test.com;link4@test.com",
        ]);
        let analyser = Analyser::new(&parsed);

        let expected = vec![
                convert_email_addresses("link1@test.com"),
                convert_email_addresses("link2@test.com"),
                convert_email_addresses("link3@test.com"),
                convert_email_addresses("link4@test.com"),
        ];

        assert_eq!(expected, analyser.sender_email_addresses().links)
    }

    #[test]
    fn sender_email_addresses_non_mailto_links() {
        let parsed = parsed_mail(vec![
            "",
            "mailto:link2@test.com;link1@test.com",
            "https://foo.bar",
        ]);
        let analyser = Analyser::new(&parsed);

        let expected = vec![
                convert_email_addresses("link1@test.com"),
                convert_email_addresses("link2@test.com"),
        ];

        assert_eq!(expected, analyser.sender_email_addresses().links)
    }

    #[test]
    fn sender_email_addresses_duplicate_addresses() {
        let parsed = parsed_mail(vec![
            "mailto:link2@test.com;link1@test.com",
            "mailto:link2@test.com;link1@test.com",
        ]);

        let analyser = Analyser::new(&parsed);

        let expected = vec![
                convert_email_addresses("link1@test.com"),
                convert_email_addresses("link2@test.com"),
        ];

        assert_eq!(expected, analyser.sender_email_addresses().links)
    }

    fn convert_email_addresses(address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Some(
                Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "test.com".into(),
                    registration_date: None,
                }
            ),
            registrar: None,
        }
    }

    fn parsed_mail(links: Vec<&str>) -> TestParsedMail {
        TestParsedMail::new(
            "from@test.com".into(),
            "reply@test.com".into(),
            "return@test.com".into(),
            "My First Phishing Email".into(),
            links,
            vec![],
            None
        )
    }
}

#[cfg(test)]
mod fulfillment_nodes_tests {
    use super::*;

    #[test]
    fn test_subject() {
        let parsed = parsed_mail(vec![]);
        let analyser = Analyser::new(&parsed);

        assert_eq!(String::from("My First Phishing Email"), analyser.subject().unwrap());
    }

    #[test]
    fn test_fullfillment_nodes() {
        let parsed = parsed_mail(
            vec![
                "https://foo.biz",
                "https://foo.baz",
                "https://foo.bar",
            ]
        );
        let analyser = Analyser::new(&parsed);

        let expected_result = vec![
            FulfillmentNode::new("https://foo.bar"),
            FulfillmentNode::new("https://foo.baz"),
            FulfillmentNode::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.fulfillment_nodes()
        )
    }

    #[test]
    fn test_fulfillment_nodes_duplicates() {
        let parsed = parsed_mail(
            vec![
                "https://foo.biz",
                "https://foo.bar",
                "https://foo.baz",
                "https://foo.bar",
            ]
        );
        let analyser = Analyser::new(&parsed);

        let expected_result = vec![
            FulfillmentNode::new("https://foo.bar"),
            FulfillmentNode::new("https://foo.baz"),
            FulfillmentNode::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.fulfillment_nodes()
        )
    }

    #[test]
    fn test_fulfillment_nodes_empty_link() {
        let parsed = parsed_mail(
            vec![
                "https://foo.biz",
                "https://foo.bar",
                "https://foo.baz",
                "",
            ]
        );
        let analyser = Analyser::new(&parsed);

        let expected_result = vec![
            FulfillmentNode::new("https://foo.bar"),
            FulfillmentNode::new("https://foo.baz"),
            FulfillmentNode::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.fulfillment_nodes()
        )
    }

    fn parsed_mail(links: Vec<&str>) -> TestParsedMail {
        TestParsedMail::new(
            "from@test.com".into(),
            "reply@test.com".into(),
            "return@test.com".into(),
            "My First Phishing Email".into(),
            links,
            vec![],
            None
        )
    }
}

#[cfg(test)]
mod delivery_nodes_tests {
    use super::*;
    use chrono::prelude::*;
    use crate::data::{DeliveryNode, HostNode};

    #[test]
    fn return_delivery_nodes() {
        let h_1 = header(
            ("a.bar.com", "b.bar.com.", "10.10.10.12"),
            "a.baz.com",
            "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)"
        );
        let h_2 = header(
            ("c.bar.com", "d.bar.com.", "10.10.10.11"),
            "b.baz.com",
            "Tue, 06 Sep 2022 16:17:21 -0700 (PDT)"
        );

        let parsed = parsed_mail(vec![&h_1, &h_2]);
        let analyser = Analyser::new(&parsed);

        let expected_result = vec![
            DeliveryNode {
                advertised_sender: Some(HostNode::new(Some("a.bar.com"), None).unwrap()),
                observed_sender: observed_sender("b.bar.com", "10.10.10.12"),
                position: 0,
                recipient: Some("a.baz.com".into()),
                time: Some(Utc.with_ymd_and_hms(2022, 9, 6, 23, 17, 22).unwrap()),
                trusted: true,
            },
            DeliveryNode {
                advertised_sender: Some(HostNode::new(Some("c.bar.com"), None).unwrap()),
                observed_sender: observed_sender("d.bar.com", "10.10.10.11"),
                position: 1,
                recipient: Some("b.baz.com".into()),
                time: Some(Utc.with_ymd_and_hms(2022, 9, 6, 23, 17, 21).unwrap()),
                trusted: false,
            },
        ];

        assert_eq!(
            expected_result, analyser.delivery_nodes("a.baz.com")
        )
    }

    #[test]
    fn marks_only_latest_node_from_trusted_recipient_as_trusted() {
        let h_1 = header(
            ("a.bar.com", "b.bar.com.", "10.10.10.12"),
            "a.baz.com",
            "Tue, 06 Sep 2022 16:17:22 -0700 (PDT)"
        );
        let h_2 = header(
            ("c.bar.com", "d.bar.com.", "10.10.10.11"),
            "b.baz.com",
            "Tue, 06 Sep 2022 16:17:21 -0700 (PDT)"
        );
        let h_3 = header(
            ("e.bar.com", "f.bar.com.", "10.10.10.10"),
            "b.baz.com",
            "Tue, 06 Sep 2022 16:17:21 -0700 (PDT)"
        );
        let h_4 = header(
            ("g.bar.com", "h.bar.com.", "10.10.10.9"),
            "c.baz.com",
            "Tue, 06 Sep 2022 16:17:21 -0700 (PDT)"
        );

        let parsed = parsed_mail(vec![&h_1, &h_2, &h_3, &h_4]);
        let analyser = Analyser::new(&parsed);

        let expected_trusted_per_position = vec![
            (0, false),
            (1, true),
            (2, false),
            (3, false),
        ];

        assert_eq!(
            expected_trusted_per_position,
            extract_trusted_by_position(analyser.delivery_nodes("b.baz.com"))
        );
    }

    fn parsed_mail(received_headers: Vec<&str>) -> TestParsedMail {
        TestParsedMail::new(
            "from@test.com".into(),
            "reply@test.com".into(),
            "return@test.com".into(),
            "My First Phishing Email".into(),
            vec![],
            received_headers,
            None,
        )
    }

    fn observed_sender(host: &str, ip_address: &str) -> Option<HostNode> {
        Some(HostNode::new(Some(host), Some(ip_address)).unwrap())
    }

    fn header(from_parts: (&str, &str, &str), by_host: &str, date: &str) -> String {
        let (advertised_host, actual_host, ip) = from_parts;

        let from = format!("{advertised_host} ({actual_host} [{ip}])");
        let by = format!("{by_host} with ESMTP id jg8-2002");
        let f_o_r = "<victim@gmail.com>";

        format!("from {from}\r\n        by {by}\r\n        for {f_o_r};\r\n        {date}")
    }

    fn extract_trusted_by_position(nodes: Vec<DeliveryNode>) -> Vec<(usize, bool)> {
        nodes
            .into_iter()
            .map(|DeliveryNode {position, trusted, ..}| (position, trusted))
            .collect()
    }
}

#[cfg(test)]
mod authentication_results_tests {
    use crate::authentication_results::{Dkim, DkimResult, Spf, SpfResult};
    use super::*;

    #[test]
    fn returns_none_if_no_authentication_results() {
        let parsed = parsed_mail(None);

        let analyser = Analyser::new(&parsed);

        assert_eq!(None, analyser.authentication_results());
    }

    #[test]
    fn returns_authentication_results() {
        let auth_header = authentication_header(dkim_portion(), spf_portion());
        let parsed = parsed_mail(Some(&auth_header));

        let analyser = Analyser::new(&parsed);

        let expected_result = AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: Some("ymy".into()),
                signature_snippet: Some("JPh8bpEm".into()),
                user_identifier_snippet: Some("@compromised.zzz".into()),
            }),
            service_identifier: Some("mx.google.com".into()),
            spf: Some(Spf {
                ip_address: Some("10.10.10.10".into()),
                result: Some(SpfResult::Pass),
                smtp_mailfrom: Some("info@xxx.fr".into()),
            })
        };

        assert_eq!(Some(expected_result), analyser.authentication_results());
    }

    fn authentication_header(dkim_portion: String, spf_portion: String) -> String {
        let provider = "mx.google.com";

        format!("{provider};\r  {dkim_portion};\r  {spf_portion}\r")
    }

    fn dkim_portion() -> String {
        "dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm".into()
    }

    fn spf_portion() -> String {
        let from = "info@xxx.fr";
        let ip = "10.10.10.10";
        let parens = format!("(google.com: domain of {from} designates {ip} as permitted sender)");

        format!("spf=pass {parens} smtp.mailfrom={from}")
    }

    fn parsed_mail(authentication_header: Option<&str>) -> TestParsedMail {
        TestParsedMail::new(
            "from@test.com".into(),
            "reply@test.com".into(),
            "return@test.com".into(),
            "My First Phishing Email".into(),
            vec![],
            vec![],
            authentication_header.map(|val| val.into())
        )
    }
// Authentication-Results: mx.google.com;\r
//        dkim=pass header.i=@compromised.zzz header.s=ymy header.b=JPh8bpEm;
//        spf=pass (google.com: domain of info@xxx.fr designates 10.10.10.10 as permitted sender) smtp.mailfrom=info@xxx.fr\r
}

#[cfg(test)]
struct TestParsedMail<'a> {
    from: String,
    reply_to: String,
    return_path: String,
    subject: String,
    links: Vec<&'a str>,
    received_headers: Vec<&'a str>,
    authentication_results: Option<String>,
}

#[cfg(test)]
impl<'a> TestParsedMail<'a> {
    fn new(
        from: String,
        reply_to: String,
        return_path: String,
        subject: String,
        links: Vec<&'a str>,
        received_headers: Vec<&'a str>,
        authentication_results: Option<String>,
    ) -> Self {
        Self {
            from,
            reply_to,
            return_path,
            subject,
            links,
            received_headers,
            authentication_results
        }
    }
}

#[cfg(test)]
impl<'a> AnalysableMessage for TestParsedMail<'a> {
    // TODO Test coverage for the below elements being missing?
    fn get_from(&self) -> Vec<String> {
        vec![self.from.clone()]
    }

    fn get_links(&self) -> Vec<String> {
        self.links.clone().into_iter().map(String::from).collect()
    }

    fn get_reply_to(&self) -> Vec<String> {
        vec![self.reply_to.clone()]
    }

    fn get_return_path(&self) -> Vec<String> {
        vec![self.return_path.clone()]
    }

    fn get_subject(&self) -> Option<String> {
        Some(self.subject.clone())
    }

    fn get_received_headers(&self) -> Vec<String> {
        self
            .received_headers
            .iter()
            .map (|header_value| String::from(*header_value))
            .collect()
    }

    fn get_authentication_results_header(&self) -> Option<String> {
        self.authentication_results.clone()
    }
}

impl<'a, T: AnalysableMessage> Analyser<'a, T> {
    pub fn new(parsed_mail: &'a T) -> Self {
        Self { parsed_mail }
    }

    pub fn subject(&self) -> Option<String> {
        self.parsed_mail.get_subject()
    }

    pub fn sender_email_addresses(&self) -> EmailAddresses {
        let pattern = Regex::new(r"\Amailto:").unwrap();

        let mut links: Vec<EmailAddressData> = self
            .parsed_mail
            .get_links()
            .iter()
            .filter(|address_string| pattern.is_match(address_string))
            .flat_map(|link| {
                if let Some((_mailto, addresses_string)) = link.split_once(':') {
                    let addresses: Vec<String> = addresses_string
                        .split(';')
                        .map(String::from)
                        .collect();
                    self.convert_addresses(addresses)
                } else {
                    vec![]
                }
            })
            .collect();

        links.sort_by(|a,b| a.address.cmp(&b.address));
        links.dedup();

        EmailAddresses {
            from: self.convert_addresses(self.parsed_mail.get_from()),
            reply_to: self.convert_addresses(self.parsed_mail.get_reply_to()),
            return_path: self.convert_addresses(self.parsed_mail.get_return_path()),
            links,
        }
    }

    pub fn delivery_nodes(&self, trusted_recipient_name: &str) -> Vec<DeliveryNode> {
        let mut trusted_node = TrustedRecipientDeliveryNode::new(trusted_recipient_name);

        self
            .parsed_mail
            .get_received_headers()
            .iter()
            .enumerate()
            .map(|(position, header_value)| {
                DeliveryNode::from_header_value(header_value, position, &mut trusted_node)
            })
            .collect()
    }

    pub fn fulfillment_nodes(&self) -> Vec<FulfillmentNode> {
        let mut nodes: Vec<FulfillmentNode> = self
            .parsed_mail
            .get_links()
            .iter()
            .filter(|link| !link.is_empty())
            .map(|url| FulfillmentNode::new(url))
            .collect();

        nodes.sort_by(|a,b| a.visible_url().cmp(b.visible_url()));
        nodes.dedup();

        nodes
    }

    pub fn authentication_results(&self) -> Option<AuthenticationResults> {
        self
            .parsed_mail
            .get_authentication_results_header()
            .map(AuthenticationResults::parse_header)
    }

    fn convert_addresses(&self, addresses: Vec<String>) -> Vec<EmailAddressData> {
        addresses
            .iter()
            .map(|addr| self.convert_address(addr))
            .collect()
    }

    fn convert_address(&self, address: &str) -> EmailAddressData {
        EmailAddressData::from_email_address(address)
    }
}
