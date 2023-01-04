use crate::data::{EmailAddressData, EmailAddresses, FulfillmentNode, Node, OutputData, Registrar};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use lettre::message::{Attachment, header::ContentType, MultiPart, SinglePart};

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
            MailDefinition::new("foo@test.com", Some("abuse@regone.zzz".into())),
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

// #[cfg(test)]
// mod send_mails_tests {
//     use super::*;
//
//     #[test]
//     fn sends_mails() {
//         let messages = vec![
//             MailDefinition::new("phishing_1@test.zzz", Some("abuse@regone.zzz")),
//             MailDefinition::new("phishing_2@test.zzz", Some("abuse@regtwo.zzz")),
//         ];
//         let from_address = "security@mydomain.com";
//         let raw_email = "Foo bar baz";
//
//     }
// }

#[derive(Debug, PartialEq)]
pub struct MailDefinition {
    entity: String,
    abuse_email_address: Option<String>
}

#[cfg(test)]
mod mail_definition_tests {
    use super::*;

    #[test]
    fn instantiation() {
        assert_eq!(
            MailDefinition {
                entity: "foo".into(),
                abuse_email_address: Some("abuse@regone.zzz".into())
            },
            MailDefinition::new("foo", Some("abuse@regone.zzz"))
        );
    }
}

impl MailDefinition {
    fn new(entity: &str, abuse_email_address: Option<&str>) -> Self {
        Self {
            entity: entity.into(),
            abuse_email_address: abuse_email_address.map(String::from)
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

    fn mailtrap_server() -> Server {
        Server::new(
            &std::env::var("TEST_SMTP_URI").unwrap(),
            &std::env::var("TEST_SMTP_USERNAME").unwrap(),
            &std::env::var("TEST_SMTP_PASSWORD").unwrap(),
        )
    }

    fn mail_definitions() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo", Some("abuse@regone.zzz".into())),
            MailDefinition::new("bar", Some("abuse@regtwo.zzz".into())),
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
            let creds = Credentials::new(String::from(&self.server.username), String::from(&self.server.password));
            let smtp_server = String::from(&self.server.host_uri);
            let from_address = String::from(&self.from_address);
            let to_address = if let Some(address) = &definition.abuse_email_address {
                String::from(address)
            } else {
                String::from("oops@test.needed")
            };
            let entity = &definition.entity;
            let subject = format!(
                "`{entity}` appears to be involved with \
                the sending of spam emails. Please investigate."
            );
            let attachment = Attachment::new(String::from("suspect_email.eml"))
                .body(String::from(raw_email), ContentType::TEXT_PLAIN);
            let body = format!(
                "Hello\n\
                I recently received a phishing email that suggests that `{entity}` \
                may be supporting \n\
                phishing. The original email is attached, can you \
                please take the appropriate action?\
                "
            );

            set.spawn(async move {

                let mailer: AsyncSmtpTransport<Tokio1Executor> =
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_server)
                    .unwrap()
                    .credentials(creds)
                    .build();

                let mail = Message::builder()
                    .from(from_address.parse().unwrap())
                    .to(to_address.parse().unwrap())
                    .subject(subject)
                    .multipart(
                        MultiPart::mixed()
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(body)
                            )
                            .singlepart(attachment)
                    ).unwrap();

                mailer.send(mail).await
            });
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap();
        }
    }
}
