use crate::analysable_message::AnalysableMessage;
use crate::data::{DeliveryNode, EmailAddressData, EmailAddresses, FulfillmentNode};
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
            links
        )
    }
}

#[cfg(test)]
mod sender_address_tests {
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
            links
        )
    }
}

#[cfg(test)]
struct TestParsedMail<'a> {
    from: String,
    reply_to: String,
    return_path: String,
    subject: String,
    links: Vec<&'a str>
}

#[cfg(test)]
impl<'a> TestParsedMail<'a> {
    fn new(
        from: String,
        reply_to: String,
        return_path: String,
        subject: String,
        links: Vec<&'a str>,
    ) -> Self {
        Self {
            from,
            reply_to,
            return_path,
            subject,
            links
        }
    }
}

#[cfg(test)]
impl<'a> AnalysableMessage for TestParsedMail<'a> {
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

    pub fn delivery_nodes(&self) -> Vec<DeliveryNode> {
        vec![]
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
