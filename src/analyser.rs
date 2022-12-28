use crate::analysable_message::AnalysableMessage;
use crate::data::{Link, EmailAddressData, EmailAddresses};
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
    fn test_links() {
        let parsed = parsed_mail(
            vec![
                "https://foo.biz",
                "https://foo.baz",
                "https://foo.bar",
            ]
        );
        let analyser = Analyser::new(&parsed);

        let expected_result = vec![
            Link::new("https://foo.bar"),
            Link::new("https://foo.baz"),
            Link::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.links()
        )
    }

    #[test]
    fn test_links_duplicates() {
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
            Link::new("https://foo.bar"),
            Link::new("https://foo.baz"),
            Link::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.links()
        )
    }

    #[test]
    fn test_links_empty_link() {
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
            Link::new("https://foo.bar"),
            Link::new("https://foo.baz"),
            Link::new("https://foo.biz"),
        ];

        assert_eq!(
            expected_result, analyser.links()
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
    fn from(&self) -> Vec<String> {
        vec![self.from.clone()]
    }

    fn links(&self) -> Vec<String> {
        self.links.clone().into_iter().map(String::from).collect()
    }

    fn reply_to(&self) -> Vec<String> {
        vec![self.reply_to.clone()]
    }

    fn return_path(&self) -> Vec<String> {
        vec![self.return_path.clone()]
    }

    fn subject(&self) -> Option<String> {
        Some(self.subject.clone())
    }
}

impl<'a, T: AnalysableMessage> Analyser<'a, T> {
    pub fn new(parsed_mail: &'a T) -> Self {
        Self { parsed_mail }
    }

    pub fn subject(&self) -> Option<String> {
        self.parsed_mail.subject()
    }

    pub fn sender_email_addresses(&self) -> EmailAddresses {

        let pattern = Regex::new(r"\Amailto:").unwrap();

        let mut links: Vec<EmailAddressData> = self
            .parsed_mail
            .links()
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
            from: self.convert_addresses(self.parsed_mail.from()),
            reply_to: self.convert_addresses(self.parsed_mail.reply_to()),
            return_path: self.convert_addresses(self.parsed_mail.return_path()),
            links,
        }
    }

    pub fn links(&self) -> Vec<Link> {
        let mut links: Vec<Link> = self
            .parsed_mail
            .links()
            .iter()
            .filter(|link| !link.is_empty())
            .map(|href| Link::new(href))
            .collect();

        links.sort_by(|a,b| a.href.cmp(&b.href));
        links.dedup();

        links
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
