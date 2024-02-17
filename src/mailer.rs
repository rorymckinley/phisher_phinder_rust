use crate::data::{
    DeliveryNode, Domain, DomainCategory, EmailAddressData, EmailAddresses, FulfillmentNode,
    HostNode, InfrastructureProvider, Node, Registrar, ReportableEntities,
};
use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

#[cfg(test)]
mod build_mail_definitions_tests {
    use super::*;
    use crate::data::{
        DeliveryNode,
        Domain,
        DomainCategory,
        EmailAddressData,
        FulfillmentNode,
        FulfillmentNodesContainer,
        HostNode,
        InfrastructureProvider,
        Node,
        ReportableEntities,
    };

    #[test]
    fn creates_definitions_for_email_addresses() {
        let actual = build_mail_definitions(Some(&input_data()));

        assert_eq!(expected(), actual);
    }

    #[test]
    fn returns_empty_collection_if_no_reportable_entities() {
        let expected: Vec<MailDefinition> = vec![];

        assert_eq!(expected, build_mail_definitions(None));
    }

    fn input_data() -> ReportableEntities {
        ReportableEntities {
            delivery_nodes: delivery_nodes(),
            email_addresses: email_addresses(),
            fulfillment_nodes_container: fulfillment_nodes(),
        }
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![email_address_data("foo@test.com", "abuse@regone.zzz")],
            links: vec![],
            reply_to: vec![],
            return_path: vec![],
        }
    }

    fn fulfillment_nodes() -> FulfillmentNodesContainer {
        FulfillmentNodesContainer {
            duplicates_removed: false,
            nodes: vec![FulfillmentNode {
                hidden: None,
                investigable: true,
                visible: Node {
                    domain: None,
                    registrar: Some(Registrar {
                        abuse_email_address: Some("abuse@regtwo.zzz".into()),
                        name: None,
                    }),
                    url: "https://dodgy.phishing.link".into(),
                },
            }]
        }
    }

    fn delivery_nodes() -> Vec<DeliveryNode> {
        vec![DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "delivery-node.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@providerone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("10.10.10.10".into()),
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }]
    }

    fn email_address_data(address: &str, abuse_email_address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: Some(abuse_email_address.into()),
                name: None,
            }),
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("foo@test.com", Some("abuse@regone.zzz")),
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regtwo.zzz")),
            MailDefinition::new("delivery-node.zzz", Some("abuse@regthree.zzz")),
            MailDefinition::new("10.10.10.10", Some("abuse@providerone.zzz")),
        ]
    }
}

pub fn build_mail_definitions(entities_option: Option<&ReportableEntities>) -> Vec<MailDefinition> {
    match entities_option {
        Some(entities) => vec![
            build_mail_definitions_from_email_addresses(&entities.email_addresses),
            build_mail_definitions_from_fulfillment_nodes(
                &entities.fulfillment_nodes_container.nodes
            ),
            build_mail_definitions_from_delivery_nodes(&entities.delivery_nodes),
        ]
        .into_iter()
        .flatten()
        .collect(),
        None => vec![],
    }
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
            ],
        }
    }

    fn email_address_data(address: &str, abuse_email_address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: Some(abuse_email_address.into()),
                name: None,
            }),
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
    ]
    .into_iter()
    .flatten()
    .collect()
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
            investigable: true,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some(abuse_email_address.into()),
                    name: None,
                }),
                url: url.into(),
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
        assert_eq!(expected(), convert_addresses_to_mail_definitions(&input()));
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
                name: None,
            }),
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("from_1@test.com", Some("abuse@regone.zzz")),
            MailDefinition::new("from_2@test.com", Some("abuse@regtwo.zzz")),
        ]
    }
}

fn convert_addresses_to_mail_definitions(
    email_addresses: &[EmailAddressData],
) -> Vec<MailDefinition> {
    email_addresses
        .iter()
        .map(convert_address_data_to_definition)
        .collect()
}

#[cfg(test)]
mod convert_address_data_to_definition_tests {
    use super::*;
    use crate::data::{Domain, DomainCategory};

    #[test]
    fn creates_mail_definition() {
        assert_eq!(expected(), convert_address_data_to_definition(&input()))
    }

    #[test]
    fn creates_mail_definition_email_provider_category() {
        assert_eq!(
            expected_email_provider_category(),
            convert_address_data_to_definition(&input_email_provider_category())
        )
    }

    #[test]
    fn creates_mail_definition_both_email_provider_and_registrar() {
        assert_eq!(
            expected_email_provider_category(),
            convert_address_data_to_definition(&input_provider_and_registrar())
        );
    }

    #[test]
    fn creates_mail_definition_email_provider_category_no_abuse_email_address() {
        assert_eq!(
            expected(),
            convert_address_data_to_definition(&input_email_provider_category_no_abuse_addr())
        )
    }

    #[test]
    fn creates_mail_definition_email_provider_category_no_abuse_email_address_no_registrar() {
        assert_eq!(
            expected_no_abuse_email_address(),
            convert_address_data_to_definition(
                &input_email_provider_category_no_abuse_addr_no_registrar()
            )
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

    fn input_email_provider_category() -> EmailAddressData {
        email_address_data("evildirtyscammer@googlemail.com", None)
    }

    fn input_provider_and_registrar() -> EmailAddressData {
        email_address_data(
            "evildirtyscammer@googlemail.com",
            Some("shouldnotseethis@regone.zzz"),
        )
    }

    fn input_email_provider_category_no_abuse_addr() -> EmailAddressData {
        EmailAddressData {
            address: "from_1@test.com".into(),
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::OpenEmailProvider,
                name: "test.com".into(),
                registration_date: None,
                resolved_domain: None,
            }),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn input_email_provider_category_no_abuse_addr_no_registrar() -> EmailAddressData {
        EmailAddressData {
            address: "from_1@test.com".into(),
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::OpenEmailProvider,
                name: "test.com".into(),
                registration_date: None,
                resolved_domain: None,
            }),
            registrar: None,
        }
    }

    fn input_no_registrar() -> EmailAddressData {
        EmailAddressData {
            address: "from_1@test.com".into(),
            domain: Domain::from_email_address("from_1@test.com"),
            registrar: None,
        }
    }

    fn email_address_data(address: &str, abuse_email_address: Option<&str>) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: Some(Registrar {
                abuse_email_address: abuse_email_address.map(String::from),
                name: None,
            }),
        }
    }

    fn expected() -> MailDefinition {
        MailDefinition::new("from_1@test.com", Some("abuse@regone.zzz"))
    }

    fn expected_no_abuse_email_address() -> MailDefinition {
        MailDefinition::new("from_1@test.com", None)
    }

    fn expected_email_provider_category() -> MailDefinition {
        MailDefinition::new("evildirtyscammer@googlemail.com", Some("abuse@gmail.com"))
    }
}

fn convert_address_data_to_definition(data: &EmailAddressData) -> MailDefinition {
    let abuse_address: Option<&str> = vec![
        extract_abuse_address_from_registrar(data.registrar.as_ref()),
        extract_abuse_address_from_domain(data.domain.as_ref()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>()
    .pop();

    MailDefinition::new(&data.address, abuse_address)
}

#[cfg(test)]
mod extract_abuse_address_from_domain_tests {
    use super::*;

    #[test]
    fn returns_none_if_domain_is_none() {
        assert!(extract_abuse_address_from_domain(None).is_none())
    }

    #[test]
    fn returns_abuse_address_if_domain_is_open_email_provider() {
        assert_eq!(
            Some("abuse@test.zzz"),
            extract_abuse_address_from_domain(Some(&email_provider_domain()))
        )
    }

    #[test]
    fn returns_abuse_address_if_domain_is_url_shortener() {
        assert_eq!(
            Some("abuse@test.zzz"),
            extract_abuse_address_from_domain(Some(&url_shortener_domain()))
        )
    }

    #[test]
    fn returns_none_if_domain_is_categorised_as_other() {
        assert_eq!(
            None,
            extract_abuse_address_from_domain(Some(&other_domain()))
        )
    }

    #[test]
    fn returns_none_if_email_provider_sans_abuse_email_address() {
        assert_eq!(
            None,
            extract_abuse_address_from_domain(Some(&email_provider_domain_no_address()))
        )
    }

    fn email_provider_domain() -> Domain {
        Domain {
            abuse_email_address: Some("abuse@test.zzz".into()),
            category: DomainCategory::OpenEmailProvider,
            name: "does-not-matter".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }

    fn url_shortener_domain() -> Domain {
        Domain {
            abuse_email_address: Some("abuse@test.zzz".into()),
            category: DomainCategory::UrlShortener,
            name: "does-not-matter".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }

    fn other_domain() -> Domain {
        Domain {
            abuse_email_address: Some("abuse@test.zzz".into()),
            category: DomainCategory::Other,
            name: "does-not-matter".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }

    fn email_provider_domain_no_address() -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::OpenEmailProvider,
            name: "does-not-matter".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }
}

fn extract_abuse_address_from_domain(domain_option: Option<&Domain>) -> Option<&str> {
    match domain_option {
        Some(domain) => match domain.category {
            DomainCategory::OpenEmailProvider | DomainCategory::UrlShortener => {
                domain.abuse_email_address.as_deref()
            }
            _ => None,
        },
        None => None,
    }
}

#[cfg(test)]
mod extract_abuse_address_from_registrar_tests {
    use super::*;

    #[test]
    fn returns_none_if_registrar_is_none() {
        assert!(extract_abuse_address_from_registrar(None).is_none())
    }

    #[test]
    fn returns_abuse_address_if_registrar() {
        assert_eq!(
            Some("abuse@regone.zzz"),
            extract_abuse_address_from_registrar(Some(&registrar_with_address()))
        )
    }

    #[test]
    fn returns_none_if_no_abuse_address() {
        assert_eq!(
            None,
            extract_abuse_address_from_registrar(Some(&registrar_without_address()))
        )
    }

    fn registrar_with_address() -> Registrar {
        Registrar {
            abuse_email_address: Some("abuse@regone.zzz".into()),
            name: None,
        }
    }

    fn registrar_without_address() -> Registrar {
        Registrar {
            abuse_email_address: None,
            name: None,
        }
    }
}

fn extract_abuse_address_from_registrar(registrar_option: Option<&Registrar>) -> Option<&str> {
    match registrar_option {
        Some(registrar) => registrar.abuse_email_address.as_deref(),
        None => None,
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
                url: "https://another.dodgy.phishing.link".into(),
            }),
            investigable: true,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regone.zzz".into()),
                    name: None,
                }),
                url: "https://dodgy.phishing.link".into(),
            },
        }
    }

    fn input_no_hidden() -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            investigable: true,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regone.zzz".into()),
                    name: None,
                }),
                url: "https://dodgy.phishing.link".into(),
            },
        }
    }

    fn expected() -> Vec<MailDefinition> {
        vec![
            MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz")),
            MailDefinition::new(
                "https://another.dodgy.phishing.link",
                Some("abuse@regtwo.zzz"),
            ),
        ]
    }

    fn expected_no_hidden() -> Vec<MailDefinition> {
        vec![MailDefinition::new(
            "https://dodgy.phishing.link",
            Some("abuse@regone.zzz"),
        )]
    }
}

fn build_mail_definitions_from_fulfillment_node(f_node: &FulfillmentNode) -> Vec<MailDefinition> {
    let mut output = vec![build_mail_definition_from_node(&f_node.visible)];

    if let Some(node) = &f_node.hidden {
        output.push(build_mail_definition_from_node(node))
    }

    output
}

#[cfg(test)]
mod build_mail_definition_from_node_tests {
    use super::*;

    #[test]
    fn build_mail_definition() {
        assert_eq!(expected(), build_mail_definition_from_node(&input()))
    }

    #[test]
    fn build_mail_definition_for_url_shortener_category() {
        assert_eq!(
            expected_url_shortener(),
            build_mail_definition_from_node(&input_url_shortener())
        )
    }

    #[test]
    fn build_mail_definition_for_registrar_and_url_shortener_category() {
        assert_eq!(
            expected_url_shortener(),
            build_mail_definition_from_node(&input_registrar_and_url_shortener())
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
            url: "https://dodgy.phishing.link".into(),
        }
    }

    fn input_url_shortener() -> Node {
        Node {
            domain: Domain::from_url("https://bit.ly/xyz"),
            registrar: None,
            url: "https://bit.ly/xyz".into(),
        }
    }

    fn input_registrar_and_url_shortener() -> Node {
        Node {
            domain: Domain::from_url("https://bit.ly/xyz"),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
            url: "https://bit.ly/xyz".into(),
        }
    }

    fn input_no_email() -> Node {
        Node {
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: None,
                name: None,
            }),
            url: "https://dodgy.phishing.link".into(),
        }
    }

    fn input_no_registrar() -> Node {
        Node {
            domain: None,
            registrar: None,
            url: "https://dodgy.phishing.link".into(),
        }
    }

    fn expected() -> MailDefinition {
        MailDefinition::new("https://dodgy.phishing.link", Some("abuse@regone.zzz"))
    }

    fn expected_url_shortener() -> MailDefinition {
        MailDefinition::new("https://bit.ly/xyz", Some("abuse@bitly.com"))
    }

    fn expected_no_abuse_email() -> MailDefinition {
        MailDefinition::new("https://dodgy.phishing.link", None)
    }
}

fn build_mail_definition_from_node(node: &Node) -> MailDefinition {
    let abuse_address = vec![
        extract_abuse_address_from_registrar(node.registrar.as_ref()),
        extract_abuse_address_from_domain(node.domain.as_ref()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>()
    .pop();

    MailDefinition::new(&node.url, abuse_address)
}

#[cfg(test)]
mod build_mail_definitions_from_delivery_nodes_tests {
    use super::*;
    use crate::data::{Domain, DomainCategory, HostNode, InfrastructureProvider};

    #[test]
    fn returns_empty_collection_if_no_delivery_nodes() {
        assert!(build_mail_definitions_from_delivery_nodes(&[]).is_empty());
    }

    #[test]
    fn returns_collection_of_delivery_node_mail_definitions() {
        let delivery_nodes = vec![delivery_node_1(), delivery_node_2()];

        let expected = vec![
            MailDefinition::new("delivery-node.zzz", Some("abuse@regthree.zzz")),
            MailDefinition::new("10.10.10.10", Some("abuse@providerone.zzz")),
        ];

        assert_eq!(
            expected,
            build_mail_definitions_from_delivery_nodes(&delivery_nodes)
        );
    }

    fn delivery_node_1() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "delivery-node.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: None,
                ip_address: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn delivery_node_2() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: None,
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@providerone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("10.10.10.10".into()),
                registrar: None,
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }
}

fn build_mail_definitions_from_delivery_nodes(
    delivery_nodes: &[DeliveryNode],
) -> Vec<MailDefinition> {
    delivery_nodes
        .iter()
        .flat_map(build_mail_definitions_from_delivery_node)
        .collect()
}

#[cfg(test)]
mod build_mail_definitions_from_delivery_node_tests {
    use super::*;
    use crate::data::{Domain, DomainCategory, HostNode, InfrastructureProvider};

    #[test]
    fn returns_empty_collection_if_no_observed_sender() {
        let node = no_observed_sender_delivery_node();

        assert!(build_mail_definitions_from_delivery_node(&node).is_empty());
    }

    #[test]
    fn returns_empty_collection_if_no_domain_and_no_ip() {
        let node = no_domain_no_ip_delivery_node();

        assert!(build_mail_definitions_from_delivery_node(&node).is_empty());
    }

    #[test]
    fn returns_mail_definitions_sans_addresses_if_no_host_or_registrar() {
        let node = no_registrar_or_infrastructure_provider_delivery_node();

        let expected = vec![
            MailDefinition::new("delivery-node.zzz", None),
            MailDefinition::new("10.10.10.10", None),
        ];

        assert_eq!(expected, build_mail_definitions_from_delivery_node(&node));
    }

    #[test]
    fn returns_mail_definition_for_host_and_domain() {
        let node = domain_and_ip_delivery_node();

        let expected = vec![
            MailDefinition::new("delivery-node.zzz", Some("abuse@regthree.zzz")),
            MailDefinition::new("10.10.10.10", Some("abuse@providerone.zzz")),
        ];

        assert_eq!(expected, build_mail_definitions_from_delivery_node(&node));
    }

    fn no_observed_sender_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: None,
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn no_domain_no_ip_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: None,
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@providerone.zzz".into()),
                    name: None,
                }),
                ip_address: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn domain_and_ip_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "delivery-node.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@providerone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("10.10.10.10".into()),
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn no_registrar_or_infrastructure_provider_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "delivery-node.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: None,
                ip_address: Some("10.10.10.10".into()),
                registrar: None,
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }
}

fn build_mail_definitions_from_delivery_node(node: &DeliveryNode) -> Vec<MailDefinition> {
    vec![
        build_mail_definition_from_delivery_node_domain(node.observed_sender.as_ref()),
        build_mail_definition_from_delivery_node_ip(node.observed_sender.as_ref()),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn build_mail_definition_from_delivery_node_domain(
    node_option: Option<&HostNode>,
) -> Option<MailDefinition> {
    match node_option {
        Some(node) => match node.domain.as_ref() {
            Some(domain) => {
                if let Some(Registrar {
                    abuse_email_address: Some(abuse_address),
                    ..
                }) = node.registrar.as_ref()
                {
                    Some(MailDefinition::new(&domain.name, Some(abuse_address)))
                } else {
                    Some(MailDefinition::new(&domain.name, None))
                }
            }
            None => None,
        },
        None => None,
    }
}

fn build_mail_definition_from_delivery_node_ip(
    node_option: Option<&HostNode>,
) -> Option<MailDefinition> {
    match node_option {
        Some(node) => match node.ip_address.as_ref() {
            Some(ip) => {
                if let Some(InfrastructureProvider {
                    abuse_email_address: Some(abuse_address),
                    ..
                }) = node.infrastructure_provider.as_ref()
                {
                    Some(MailDefinition::new(ip, Some(abuse_address)))
                } else {
                    Some(MailDefinition::new(ip, None))
                }
            }
            None => None,
        },
        None => None,
    }
}

#[derive(Debug, PartialEq)]
pub struct MailDefinition {
    entity: Entity,
    abuse_email_address: Option<String>,
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
                entity: Entity::Node(url.into()),
                abuse_email_address: Some("abuse@regone.zzz".into())
            },
            MailDefinition::new("https://foo.bar.baz/", Some("abuse@regone.zzz"))
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
        let entity = match Url::parse(entity) {
            Ok(_) => Entity::Node(entity.into()),
            Err(_) => Entity::EmailAddress(entity.into()),
        };

        Self {
            entity,
            abuse_email_address: abuse_email_address.map(String::from),
        }
    }

    fn reportable(&self) -> bool {
        match &self.entity {
            Entity::Node(url_string) => {
                //TODO Add error handling here
                let url = Url::parse(url_string).unwrap();
                url.scheme() == "https" || url.scheme() == "http"
            },
            Entity::EmailAddress(_) => true,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Entity {
    EmailAddress(String),
    Node(String),
}

#[cfg(test)]
mod entity_tests {
    use super::*;

    #[test]
    fn email_variant_as_string() {
        let entity = Entity::EmailAddress("foo@test.com".into());

        assert_eq!(String::from("foo@test.com"), entity.to_string());
    }

    #[test]
    fn url_variant_as_string() {
        let url = "https://foo.bar.baz.com/fuzzy/wuzzy";

        let entity = Entity::Node(url::Url::parse(url).unwrap().into());

        assert_eq!(String::from(url), entity.to_string());
    }

    #[test]
    fn noisy_url_variant_as_string() {
        let url = "https://user:secret@foo.bar.baz.com:1234/fuzzy/wuzzy?blah=meh#xyz";
        let expected_url = "https://foo.bar.baz.com/fuzzy/wuzzy";

        let entity = Entity::Node(url::Url::parse(url).unwrap().into());

        assert_eq!(String::from(expected_url), entity.to_string());
    }

    #[test]
    fn url_variant_without_host_as_string() {
        let url = "file:///foo/bar";

        let entity = Entity::Node(url::Url::parse(url).unwrap().into());

        assert_eq!(String::from(url), entity.to_string());
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::EmailAddress(email_address) => {
                write!(f, "{email_address}")
            }
            Entity::Node(url_string) => {
                // TODO Better error handling
                let url = Url::parse(url_string).unwrap();
                let host = url.host_str().unwrap_or("");
                let display_url = format!(
                    "{}://{}{}",
                    url.scheme(),
                    host,
                    url.path()
                );
                write!(f, "{display_url}")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Server {
    host_uri: String,
    password: String,
    username: String,
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
            password: password.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Mailer {
    server: Server,
    from_address: String,
}

#[cfg(test)]
mod mailer_tests {
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
    #[ignore]
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
                &raw_email(),
            ),
            Email::new(
                "from@test.com",
                "abuse@regtwo.zzz",
                &mail_subject("bar"),
                &mail_body("bar"),
                &raw_email(),
            ),
        ]);

        assert_eq!(
            expected,
            sorted_mail_trap_records(mailtrap.get_all_emails())
        );
    }

    #[test]
    #[ignore]
    fn does_not_send_emails_if_no_abuse_address() {
        let mailtrap = initialise_mail_trap();

        let mailer = Mailer::new(mailtrap_server(), "from@test.com");

        tokio_test::block_on(
            mailer.send_mails(&mail_definitions_including_no_abuse_contact(), &raw_email()),
        );

        let expected = sorted_mail_trap_records(vec![Email::new(
            "from@test.com",
            "abuse@regone.zzz",
            &mail_subject("foo"),
            &mail_body("foo"),
            &raw_email(),
        )]);

        assert_eq!(
            expected,
            sorted_mail_trap_records(mailtrap.get_all_emails())
        );
    }

    #[test]
    #[ignore]
    fn does_not_send_mails_if_reportable_entity_is_not_reportable() {
        let mailtrap = initialise_mail_trap();

        let mailer = Mailer::new(mailtrap_server(), "from@test.com");

        tokio_test::block_on(mailer.send_mails(
            &mail_definitions_including_no_reportable_entity(),
            &raw_email(),
        ));

        let expected = sorted_mail_trap_records(vec![Email::new(
            "from@test.com",
            "abuse@regone.zzz",
            &mail_subject("foo"),
            &mail_body("foo"),
            &raw_email(),
        )]);

        assert_eq!(
            expected,
            sorted_mail_trap_records(mailtrap.get_all_emails())
        );
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
        emails.sort_by(|a, b| a.to.cmp(&b.to));
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
            from_address: from_address.into(),
        }
    }

    pub async fn send_mails(&self, definitions: &[MailDefinition], raw_email: &str) {
        use tokio::task::JoinSet;

        let mut set: JoinSet<
            Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error>,
        > = JoinSet::new();
        for definition in definitions.iter() {
            if definition.reportable() {
                if let Some(abuse_email_address) = &definition.abuse_email_address {
                    let mailer = self.build_mailer();

                    let mail = self.build_mail(abuse_email_address, &definition.entity, raw_email);

                    set.spawn(async move { mailer.send(mail).await });
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
                    .singlepart(self.build_attachment(raw_email)),
            )
            .unwrap()
    }

    fn credentials(&self) -> Credentials {
        Credentials::new(
            String::from(&self.server.username),
            String::from(&self.server.password),
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
