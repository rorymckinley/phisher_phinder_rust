use crate::data::{EmailAddressData, EmailAddresses, FulfillmentNode, Node, OutputData, Registrar};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use lettre::message::{Attachment, header::ContentType, MultiPart, SinglePart};
use url::Url;
use std::fmt;

#[cfg(test)]
mod build_mail_definitions_tests {
    use super::*;
    use crate::data::{
        Domain, EmailAddressData, FulfillmentNode, Node, ParsedMail
    };

    #[test]
    fn creates_definitions_for_email_addresses() {
        let actual = build_mail_definitions(&input_data());

        assert_eq!(expected(), actual);
    }

    fn input_data() -> OutputData {
        let parsed_mail = ParsedMail::new(
            email_addresses(),
            vec![],
            fulfillment_nodes(),
            Some("".into())
        );

        OutputData::new(parsed_mail, "")
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![email_address_data("foo@test.com", "abuse@regone.zzz")],
            links: vec![],
            reply_to: vec![],
            return_path: vec![]
        }
    }

    fn fulfillment_nodes() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode {
                hidden: None,
                visible: Node {
                    domain: None,
                    registrar: Some(Registrar {
                        abuse_email_address: Some("abuse@regtwo.zzz".into()),
                        name: None,
                    }),
                    url: "https://dodgy.phishing.link".into()
                },
            }
        ]
    }

    fn email_address_data(address: &str, abuse_email_address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: Some(abuse_email_address.into()),
                name: None
            })
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo@test.com", Some("abuse@regone.zzz")),
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regtwo.zzz"))
        ]
    }
}

pub fn build_mail_definitions(data: &OutputData) -> Vec<MailDefinition> {
    vec![
        build_mail_definitions_from_email_addresses(&data.parsed_mail.email_addresses),
        build_mail_definitions_from_fulfillment_nodes(&data.parsed_mail.fulfillment_nodes)
    ].into_iter().flatten().collect()
}

#[cfg(test)]
mod build_mail_definitions_from_email_addresses_tests {
    use super::*;
    use crate::data::{Domain, EmailAddressData, Registrar};

    #[test]
    fn generates_definitions_for_all_email_addresses() {
        assert_eq!(
            expected(),
            build_mail_definitions_from_email_addresses(&input())
        );
    }

    fn input() -> EmailAddresses {
        EmailAddresses {
            from: vec![
                email_address_data("from_1@test.com", "abuse@regone.zzz"),
                email_address_data("from_2@test.com", "abuse@regtwo.zzz"),
            ],
            links: vec![
                email_address_data("links_1@test.com", "abuse@regthree.zzz"),
                email_address_data("links_2@test.com", "abuse@regfour.zzz"),
            ],
            reply_to: vec![
                email_address_data("reply_to_1@test.com", "abuse@regfive.zzz"),
                email_address_data("reply_to_2@test.com", "abuse@regsix.zzz"),
            ],
            return_path: vec![
                email_address_data("return_path_1@test.com", "abuse@regseven.zzz"),
                email_address_data("return_path_2@test.com", "abuse@regeight.zzz"),
            ]
        }
    }

    fn email_address_data(address: &str, abuse_email_address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: Some(abuse_email_address.into()),
                name: None
            })
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("from_1@test.com", Some("abuse@regone.zzz")),
            MailDefinition::new("from_2@test.com", Some("abuse@regtwo.zzz")),
            MailDefinition::new("links_1@test.com", Some("abuse@regthree.zzz")),
            MailDefinition::new("links_2@test.com", Some("abuse@regfour.zzz")),
            MailDefinition::new("reply_to_1@test.com", Some("abuse@regfive.zzz")),
            MailDefinition::new("reply_to_2@test.com", Some("abuse@regsix.zzz")),
            MailDefinition::new("return_path_1@test.com", Some("abuse@regseven.zzz")),
            MailDefinition::new("return_path_2@test.com", Some("abuse@regeight.zzz")),
        ]
    }
}

fn build_mail_definitions_from_email_addresses(addresses: &EmailAddresses) -> Vec<MailDefinition> {
    vec![
        convert_addresses_to_mail_definitions(&addresses.from),
        convert_addresses_to_mail_definitions(&addresses.links),
        convert_addresses_to_mail_definitions(&addresses.reply_to),
        convert_addresses_to_mail_definitions(&addresses.return_path),
    ].into_iter().flatten().collect()
}

#[cfg(test)]
mod build_mail_definitions_from_fulfillment_nodes_tests {
    use super::*;
    use crate::data::Node;

    #[test]
    fn builds_mail_definitions() {
        assert_eq!(
            expected(),
            build_mail_definitions_from_fulfillment_nodes(&input())
        )
    }

    fn input() -> Vec<FulfillmentNode> {
        vec![
            fulfillment_node("https://dodgy.phishing.link", "abuse@regone.zzz"),
            fulfillment_node("https://also.dodgy.phishing.link", "abuse@regtwo.zzz"),
        ]
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz")),
            MailDefinition::new("https://also.dodgy.phishing.link", Some("abuse@regtwo.zzz")),
        ]
    }

    fn fulfillment_node(url: &str, abuse_email_address: &str) -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some(abuse_email_address.into()),
                    name: None,
                }),
                url: url.into()
            },
        }
    }
}

fn build_mail_definitions_from_fulfillment_nodes(nodes: &[FulfillmentNode]) -> Vec<MailDefinition> {
    nodes
        .iter()
        .flat_map(build_mail_definitions_from_fulfillment_node)
        .collect()
}

#[cfg(test)]
mod convert_addresses_to_mail_definitions_tests {
    use super::*;
    use crate::data::Domain;

    #[test]
    fn converts_collection_of_email_address_data() {
        assert_eq!(
            expected(),
            convert_addresses_to_mail_definitions(&input())
        );
    }

    fn input() -> Vec<EmailAddressData> {
        vec![
            email_address_data("from_1@test.com", "abuse@regone.zzz"),
            email_address_data("from_2@test.com", "abuse@regtwo.zzz"),
        ]
    }

    fn email_address_data(address: &str, abuse_email_address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: Some(abuse_email_address.into()),
                name: None
            })
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("from_1@test.com", Some("abuse@regone.zzz")),
            MailDefinition::new("from_2@test.com", Some("abuse@regtwo.zzz")),
        ]
    }
}

fn convert_addresses_to_mail_definitions(email_addresses: &[EmailAddressData]) -> Vec<MailDefinition> {
    email_addresses.iter().map(|e_a_d| {
        convert_address_data_to_definition(e_a_d)
    }).collect()
}

#[cfg(test)]
mod convert_address_data_to_definition_tests {
    use super::*;
    use crate::data::Domain;

    #[test]
    fn creates_mail_definition() {
        assert_eq!(
            expected(),
            convert_address_data_to_definition(&input())
        )
    }

    #[test]
    fn creates_mail_definition_no_abuse_email() {
        assert_eq!(
            expected_no_abuse_email_address(),
            convert_address_data_to_definition(&input_no_abuse_email_address())
        )
    }

    #[test]
    fn creates_mail_definition_no_registrar() {
        assert_eq!(
            expected_no_abuse_email_address(),
            convert_address_data_to_definition(&input_no_registrar())
        )
    }

    fn input() -> EmailAddressData {
        email_address_data("from_1@test.com", Some("abuse@regone.zzz"))
    }

    fn input_no_abuse_email_address() -> EmailAddressData {
        email_address_data("from_1@test.com", None)
    }

    fn input_no_registrar() -> EmailAddressData {
        EmailAddressData {
            address: "from_1@test.com".into(),
            domain: Domain::from_email_address("from_1@test.com"),
            registrar: None
        }
    }

    fn email_address_data(address: &str, abuse_email_address: Option<&str>) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: abuse_email_address.map(String::from),
                name: None
            })
        }
    }

    fn expected() -> MailDefinition {
        MailDefinition::new("from_1@test.com", Some("abuse@regone.zzz"))
    }

    fn expected_no_abuse_email_address() -> MailDefinition {
        MailDefinition::new("from_1@test.com", None)
    }
}

fn convert_address_data_to_definition(data: &EmailAddressData) -> MailDefinition {
    if let Some(Registrar { abuse_email_address, .. }) = &data.registrar {
        MailDefinition::new(&data.address, abuse_email_address.as_deref())
    } else {
        MailDefinition::new(&data.address, None)
    }
}

#[cfg(test)]
mod build_mail_definitions_from_fulfillment_node_tests {
    use super::*;
    use crate::data::Node;

    #[test]
    fn returns_definitions_for_visible_and_hidden_nodes() {
        assert_eq!(
            expected(),
            build_mail_definitions_from_fulfillment_node(&input())
        )
    }

    #[test]
    fn returns_definitions_for_visible_but_no_hidden_node() {
        assert_eq!(
            expected_no_hidden(),
            build_mail_definitions_from_fulfillment_node(&input_no_hidden())
        )
    }

    fn input() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regtwo.zzz".into()),
                    name: None,
                }),
                url: "https://another.dodgy.phishing.link".into()
            }),
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regone.zzz".into()),
                    name: None,
                }),
                url: "https://dodgy.phishing.link".into()
            },
        }
    }

    fn input_no_hidden() -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regone.zzz".into()),
                    name: None,
                }),
                url: "https://dodgy.phishing.link".into()
            },
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz")),
            MailDefinition::new("https://another.dodgy.phishing.link", Some("abuse@regtwo.zzz")),
        ]
    }

    fn expected_no_hidden() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz")),
        ]
    }
}

fn build_mail_definitions_from_fulfillment_node(f_node: &FulfillmentNode) ->  Vec<MailDefinition> {
    let mut output = vec![
        build_mail_definition_from_node(&f_node.visible)
    ];

    if let Some(node) = &f_node.hidden {
        output.push(build_mail_definition_from_node(node))
    }

    output
}

#[cfg(test)]
mod build_mail_definition_from_node_tests {
    use  super::*;

    #[test]
    fn build_mail_definition() {
        assert_eq!(
            expected(),
            build_mail_definition_from_node(&input())
        )
    }

    #[test]
    fn build_mail_definition_no_abuse_email() {
        assert_eq!(
            expected_no_abuse_email(),
            build_mail_definition_from_node(&input_no_email())
        )
    }

    #[test]
    fn build_mail_definition_no_registrar() {
        assert_eq!(
            expected_no_abuse_email(),
            build_mail_definition_from_node(&input_no_registrar())
        )
    }

    fn input() -> Node {
        Node {
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
            url: "https://dodgy.phishing.link".into()
        }
    }

    fn input_no_email() -> Node {
        Node {
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: None,
                name: None,
            }),
            url: "https://dodgy.phishing.link".into()
        }
    }

    fn input_no_registrar() -> Node {
        Node {
            domain: None,
            registrar: None,
            url: "https://dodgy.phishing.link".into()
        }
    }

    fn expected() -> MailDefinition {
        MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz"))
    }

    fn expected_no_abuse_email() -> MailDefinition {
        MailDefinition::new("https://dodgy.phishing.link", None)
    }
}

fn build_mail_definition_from_node(node: &Node) -> MailDefinition {
    if let Some(Registrar {abuse_email_address: Some(abuse_email_address), ..}) = &node.registrar {
        MailDefinition::new(&node.url, Some(abuse_email_address))
    } else {
        MailDefinition::new(&node.url, None)
    }
}

#[derive(Debug, PartialEq)]
pub struct MailDefinition {
    entity: Entity,
    abuse_email_address: Option<String>
}

#[cfg(test)]
mod mail_definition_tests {
    use super::*;

    #[test]
    fn instantiation_with_email_address() {
        assert_eq!(
            MailDefinition {
                entity: Entity::EmailAddress("foo@test.zzz".into()),
                abuse_email_address: Some("abuse@regone.zzz".into())
            },
            MailDefinition::new("foo@test.zzz", Some("abuse@regone.zzz"))
        );
    }

    #[test]
    fn instantiation_with_url() {
        let url = Url::parse("https://foo.bar.baz").unwrap();

        assert_eq!(
            MailDefinition {
                entity: Entity::FulfillmentNode(url),
                abuse_email_address: Some("abuse@regone.zzz".into())
            },
            MailDefinition::new("https://foo.bar.baz", Some("abuse@regone.zzz"))
        );
    }

    #[test]
    fn is_reportable_if_entity_is_email_address() {
        let definition = MailDefinition::new("a@test.com", Some("abuse@regone.zzz"));

        assert!(definition.reportable());
    }

    #[test]
    fn is_reportable_if_entity_is_url_and_protocol_is_https() {
        let definition = MailDefinition::new("https://foo.bar.baz", Some("abuse@regone.zzz"));

        assert!(definition.reportable());
    }

    #[test]
    fn is_reportable_if_entity_is_url_and_protocol_is_http() {
        let definition = MailDefinition::new("http://foo.bar.baz", Some("abuse@regone.zzz"));

        assert!(definition.reportable());
    }

    #[test]
    fn is_not_reportable_if_entity_is_url_and_protocol_is_neither_http_not_https() {
        let definition = MailDefinition::new("file:///foo/bar", Some("abuse@regone.zzz"));

        assert!(!definition.reportable());
    }
}

impl MailDefinition {
    fn new(entity: &str, abuse_email_address: Option<&str>) -> Self {
        let entity = match  Url::parse(entity) {
            Ok(url) => Entity::FulfillmentNode(url),
            Err(_) => Entity::EmailAddress(entity.into())
        };

        Self {
            entity,
            abuse_email_address: abuse_email_address.map(String::from)
        }
    }

    fn reportable(&self) -> bool {
        match &self.entity {
            Entity::FulfillmentNode(url) => {
                url.scheme() == "https" || url.scheme() == "http"
            },
            Entity::EmailAddress(_) => true
        }
    }
}

#[derive(Debug, PartialEq)]
enum Entity {
    EmailAddress(String),
    FulfillmentNode(Url)
}

#[cfg(test)]
mod entity_tests {
    use super::*;

    #[test]
    fn email_variant_as_string() {
        let entity = Entity::EmailAddress("foo@test.com".into());

        assert_eq!(
            String::from("foo@test.com"),
            entity.to_string()
        );
    }

    #[test]
    fn url_variant_as_string() {
        let url = "https://foo.bar.baz.com/fuzzy/wuzzy";

        let entity = Entity::FulfillmentNode(url::Url::parse(url).unwrap());

        assert_eq!(
            String::from(url),
            entity.to_string()
        );
    }

    #[test]
    fn noisy_url_variant_as_string() {
        let url = "https://user:secret@foo.bar.baz.com:1234/fuzzy/wuzzy?blah=meh#xyz";
        let expected_url = "https://foo.bar.baz.com/fuzzy/wuzzy";

        let entity = Entity::FulfillmentNode(url::Url::parse(url).unwrap());

        assert_eq!(
            String::from(expected_url),
            entity.to_string()
        );
    }

    #[test]
    fn url_variant_without_host_as_string() {
        let url = "file:///foo/bar";

        let entity = Entity::FulfillmentNode(url::Url::parse(url).unwrap());

        assert_eq!(
            String::from(url),
            entity.to_string()
        );
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::EmailAddress(email_address) => {
                write!(f, "{email_address}")
            },
            Entity::FulfillmentNode(original_url) => {
                let host = original_url.host_str().unwrap_or("");
                let url = format!("{}://{}{}", original_url.scheme(), host, original_url.path());
                write!(f, "{url}")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Server {
    host_uri: String,
    password: String,
    username: String
}

#[cfg(test)]
mod server_tests {
    use super::*;

    #[test]
    fn instantiates_itself() {
        assert_eq!(
            Server {
                host_uri: "foo.test.com".into(),
                username: "my_user".into(),
                password: "my_secret".into()
            },
            Server::new("foo.test.com", "my_user", "my_secret")
        );
    }
}

impl Server {
    pub fn new(host_uri: &str, username: &str, password: &str) -> Self {
        Self {
            host_uri: host_uri.into(),
            username: username.into(),
            password: password.into()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Mailer {
    server: Server,
    from_address: String
}

#[cfg(test)]
mod  mailer_tests {
    use super::*;
    use crate::mail_trap::{Email, MailTrap};

    #[test]
    fn instantiates_itself() {
        let server = Server::new("foo.test.com", "my_user", "my_secret");

        assert_eq!(
            Mailer {
                server: Server::new("foo.test.com", "my_user", "my_secret"),
                from_address: "from@test.com".into()
            },
            Mailer::new(server, "from@test.com")
        );
    }

    #[test]
    fn it_sends_emails() {
        let mailtrap = initialise_mail_trap();

        let mailer = Mailer::new(mailtrap_server(), "from@test.com");

        tokio_test::block_on(mailer.send_mails(&mail_definitions(), &raw_email()));

        let expected = sorted_mail_trap_records(vec![
            Email::new(
                "from@test.com",
                "abuse@regone.zzz",
                &mail_subject("foo"),
                &mail_body("foo"),
                &raw_email()
            ),
            Email::new(
                "from@test.com",
                "abuse@regtwo.zzz",
                &mail_subject("bar"),
                &mail_body("bar"),
                &raw_email()
            )
        ]);

        assert_eq!(expected, sorted_mail_trap_records(mailtrap.get_all_emails()));
    }

    #[test]
    fn does_not_send_emails_if_no_abuse_address() {
        let mailtrap = initialise_mail_trap();

        let mailer = Mailer::new(mailtrap_server(), "from@test.com");

        tokio_test::block_on(
            mailer.send_mails(&mail_definitions_including_no_abuse_contact(), &raw_email())
        );

        let expected = sorted_mail_trap_records(vec![
            Email::new(
                "from@test.com",
                "abuse@regone.zzz",
                &mail_subject("foo"),
                &mail_body("foo"),
                &raw_email()
            )
        ]);

        assert_eq!(expected, sorted_mail_trap_records(mailtrap.get_all_emails()));
    }

    #[test]
    fn does_not_send_mails_if_reportable_entity_is_not_reportable() {
        let mailtrap = initialise_mail_trap();

        let mailer = Mailer::new(mailtrap_server(), "from@test.com");

        tokio_test::block_on(
            mailer.send_mails(&mail_definitions_including_no_reportable_entity(), &raw_email())
        );

        let expected = sorted_mail_trap_records(vec![
            Email::new(
                "from@test.com",
                "abuse@regone.zzz",
                &mail_subject("foo"),
                &mail_body("foo"),
                &raw_email()
            )
        ]);

        assert_eq!(expected, sorted_mail_trap_records(mailtrap.get_all_emails()));
    }

    fn mailtrap_server() -> Server {
        Server::new(
            &std::env::var("TEST_SMTP_URI").unwrap(),
            &std::env::var("TEST_SMTP_USERNAME").unwrap(),
            &std::env::var("TEST_SMTP_PASSWORD").unwrap(),
        )
    }

    fn mail_definitions() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo", Some("abuse@regone.zzz")),
            MailDefinition::new("bar", Some("abuse@regtwo.zzz")),
        ]
    }

    fn mail_definitions_including_no_abuse_contact() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo", Some("abuse@regone.zzz")),
            MailDefinition::new("bar", None),
        ]
    }

    fn mail_definitions_including_no_reportable_entity() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo", Some("abuse@regone.zzz")),
            MailDefinition::new("file:///foo/bar", Some("abuse@regtwo.zzz")),
        ]
    }

    fn raw_email() -> String {
        "Foo, Bar, Baz".into()
    }

    fn initialise_mail_trap() -> MailTrap {
        let mail_trap = MailTrap::new(mail_trap_api_token());

        mail_trap.clear_mails();

        mail_trap
    }

    fn mail_trap_api_token() -> String {
        std::env::var("MAILTRAP_API_TOKEN").unwrap()
    }

    fn sorted_mail_trap_records(mut emails: Vec<Email>) -> Vec<Email> {
       emails.sort_by(|a,b| a.to.cmp(&b.to));
       emails
    }

    fn mail_subject(entity: &str) -> String {
        format!(
            "`{entity}` appears to be involved with the sending of spam emails. Please investigate."
        )
    }

    fn mail_body(entity: &str) -> String {
        format!(
            "Hello\n\
            I recently received a phishing email that suggests that `{entity}` may be supporting \n\
            phishing. The original email is attached, can you please take the appropriate action?\
            "
        )
    }
}

impl Mailer {
    pub fn new(server: Server, from_address: &str) -> Self {
        Self {
            server,
            from_address: from_address.into()
        }
    }

    pub async fn send_mails(&self, definitions: &[MailDefinition], raw_email: &str)  {
        use tokio::task::JoinSet;

        let mut set: JoinSet<Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error>> = JoinSet::new();
        for definition in definitions.iter() {
            if definition.reportable() {
                if let Some(abuse_email_address) = &definition.abuse_email_address {

                    let mailer = self.build_mailer();

                    let mail = self.build_mail(abuse_email_address, &definition.entity, raw_email);

                    set.spawn(async move {
                        mailer.send(mail).await
                    });
                }
            }
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap();
        }
    }

    fn build_mail(&self, abuse_email_address: &str, entity: &Entity, raw_email: &str) -> Message {
        // TODO find some way to exercise these `unwrap()` calls and put better 
        // handling in
        Message::builder()
            .from(self.from_address.parse().unwrap())
            .to(abuse_email_address.parse().unwrap())
            .subject(self.build_subject(entity))
            .multipart(
                MultiPart::mixed()
                    .singlepart(self.build_body(entity))
                    .singlepart(self.build_attachment(raw_email))
            ).unwrap()
    }

    fn credentials(&self) -> Credentials {
        Credentials::new(
            String::from(&self.server.username),
            String::from(&self.server.password)
        )
    }

    fn build_subject(&self, entity: &Entity) -> String {
        format!(
            "`{entity}` appears to be involved with \
            the sending of spam emails. Please investigate."
        )
    }

    fn build_body(&self, entity: &Entity) -> SinglePart {
        let text = format!(
            "\
            Hello\n\
            I recently received a phishing email that suggests that `{entity}` \
            may be supporting \n\
            phishing. The original email is attached, can you \
            please take the appropriate action?\
            "
        );

        SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body(text)
    }

    fn build_attachment(&self, raw_email: &str) -> SinglePart {
        Attachment::new(String::from("suspect_email.eml"))
            .body(String::from(raw_email), ContentType::TEXT_PLAIN)
    }

    fn build_mailer(&self) -> AsyncSmtpTransport<Tokio1Executor> {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.server.host_uri)
            .unwrap()
            .credentials(self.credentials())
            .build()
    }
}
