use crate::data::{
    DeliveryNode,
    Domain,
    EmailAddressData,
    EmailAddresses,
    FulfillmentNode,
    HostNode,
    InfrastructureProvider,
    Node,
    OutputData,
    Registrar
};
use crate::mailer::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Notification {
    Email(Entity, String)
}

impl Notification {
    pub fn via_email(entity: Entity, recipient: String) -> Self {
        Self::Email(entity, recipient)
    }
}

pub fn add_notifications(data: OutputData) -> OutputData {
    if let Some(entities) = &data.reportable_entities {
        let notifications = vec![
            build_notifications_from_email_addresses(&entities.email_addresses),
            build_notifications_from_fulfillment_nodes(
                &entities.fulfillment_nodes_container.nodes
            ),
            build_notifications_from_delivery_nodes(&entities.delivery_nodes),
        ]
            .into_iter()
            .flatten()
            .collect();

        OutputData {
            notifications,
            ..data
        }
    } else {
        data
    }
}

fn build_notifications_from_email_addresses(addresses: &EmailAddresses) -> Vec<Notification> {
    [
        to_refs(&addresses.from),
        to_refs(&addresses.links),
        to_refs(&addresses.reply_to),
        to_refs(&addresses.return_path)
    ]
    .iter()
    .flatten()
    .filter_map(|address_data| build_notification_from_email_address(address_data))
    .collect()
}

fn to_refs(data: &[EmailAddressData]) -> Vec<&EmailAddressData> {
    data.iter().collect()
}

fn build_notification_from_email_address(data: &EmailAddressData) -> Option<Notification> {
    let entity = Entity::EmailAddress(data.address.clone());
    if let Some(domain) = data.domain.as_ref() {
        let domain_notification = build_notification_for_domain(
            domain, Entity::EmailAddress(data.address.clone())
        );

        if domain_notification.is_some() {
            domain_notification
        } else {
            build_notification_for_registrar(data.registrar.as_ref(), entity)
        }
    } else {
        build_notification_for_registrar(data.registrar.as_ref(), entity)
    }
}

fn build_notifications_from_fulfillment_nodes(nodes: &[FulfillmentNode]) -> Vec<Notification> {
    nodes
        .iter()
        .flat_map(build_notifications_from_fulfillment_node)
        .collect()
}

fn build_notifications_from_fulfillment_node(f_node: &FulfillmentNode) -> Vec<Notification> {
    vec![
        build_notification_for_node(f_node.hidden.as_ref()),
        build_notification_for_node(Some(&f_node.visible)),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn build_notification_for_node(node_option: Option<&Node>) -> Option<Notification> {
    if let Some(node) = node_option {
        if let Some(domain) = node.domain.as_ref() {
            match build_notification_for_domain(domain, build_node_entity(&node.url)) {
                None => {
                    build_notification_for_registrar(
                        node.registrar.as_ref(), build_node_entity(&node.url)
                    )
                }
                notification => notification,
            }
        } else {
            build_notification_for_registrar(
                node.registrar.as_ref(), build_node_entity(&node.url)
            )
        }
    } else {
        None
    }
}

fn build_notifications_from_delivery_nodes(nodes: &[DeliveryNode]) -> Vec<Notification> {
    nodes
        .iter()
        .flat_map(build_notifications_for_delivery_node)
        .collect()
}

fn build_notifications_for_delivery_node(node: &DeliveryNode) -> Vec<Notification> {
    if let Some(sender) = &node.observed_sender {
        [
            build_notification_for_delivery_node_domain(sender),
            build_notification_for_delivery_node_ip(sender),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        vec![]
    }
}

fn build_notification_for_delivery_node_domain(sender: &HostNode) -> Option<Notification> {
    if let Some(domain) = &sender.domain {
        let entity = Entity::Node(domain.name.clone());

        build_notification_for_registrar(sender.registrar.as_ref(), entity)
    } else {
        None
    }
}

fn build_notification_for_delivery_node_ip(sender: &HostNode) -> Option<Notification> {
    if let Some(InfrastructureProvider {
        abuse_email_address: Some(address),
        ..
    }) = sender.infrastructure_provider.as_ref() {
        sender.ip_address.as_ref().map(|ip_address| {
            Notification::via_email(Entity::Node(ip_address.into()), address.into())
        })
    } else {
        None
    }
}

fn build_notification_for_registrar(
    registrar_option: Option<&Registrar>, entity: Entity
) -> Option<Notification> {
    match registrar_option {
        Some(registrar) => {
            registrar.abuse_email_address.as_ref().map(|address| {
                Notification::via_email(entity, address.into())
            })
        },
        None => None
    }
}

fn build_notification_for_domain(domain: &Domain, entity: Entity) -> Option<Notification> {
    domain
        .abuse_email_address
        .as_ref()
        .map(|address| Notification::via_email(entity, address.into()))
}

fn build_node_entity(url: &str) -> Entity {
    Entity::Node(url.into())
}


#[cfg(test)]
mod notification_email_tests {
    use super::*;

    #[test]
    fn returns_a_notification_email_enum() {
        let entity = Entity::EmailAddress("a@b.com".into());
        let recipient = String::from("abuse@registrarone.com");

        let notification = Notification::via_email(
            Entity::EmailAddress("a@b.com".into()),
            String::from("abuse@registrarone.com")
        );

        assert_eq!(notification, Notification::Email(entity, recipient));
    }
}

#[cfg(test)]
mod add_notifications_tests {
    use crate::data::{
        DeliveryNode,
        Domain,
        DomainCategory,
        EmailAddresses,
        EmailAddressData,
        FulfillmentNode,
        FulfillmentNodesContainer,
        HostNode,
        InfrastructureProvider,
        Node,
        ParsedMail,
        Registrar,
        ReportableEntities
    };
    use crate::message_source::MessageSource;
    use super::*;

    #[test]
    fn adds_a_notifications_for_reportable_entities() {
       let data = build_output_data();

       let data_with_notifications = add_notifications(data);

       assert_eq!(
           expected_output_data(build_output_data()),
           data_with_notifications
       );
    }

    #[test]
    fn returns_empty_list_if_no_reportable_entities() {
       let data = build_output_data_sans_entities();

       let data_with_notifications = add_notifications(data);

       let expected: Vec<Notification> = vec![];

       assert_eq!(expected, data_with_notifications.notifications);
    }

    fn build_output_data() -> OutputData {
        OutputData {
            parsed_mail: parsed_mail(),
            message_source: MessageSource::new(""),
            notifications: vec![],
            reportable_entities: Some(ReportableEntities {
                delivery_nodes: build_delivery_nodes(),
                email_addresses: reportable_email_addresses(),
                fulfillment_nodes_container: fulfillment_nodes(),
            }),
            run_id: None
        }
    }

    fn build_output_data_sans_entities() -> OutputData {
        OutputData {
            parsed_mail: parsed_mail(),
            message_source: MessageSource::new(""),
            notifications: vec![],
            reportable_entities: None,
            run_id: None
        }
    }

    fn expected_output_data(input_data: OutputData) -> OutputData {
        OutputData {
            notifications: expected_notifications(),
            ..input_data
        }
    }

    fn expected_notifications() -> Vec<Notification> {
        vec![
            Notification::via_email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@regone.zzz".into()
            ),
            Notification::via_email(
                Entity::Node("https://dodgy.phishing.link".into()),
                "abuse@regtwo.zzz".into()
            ),
            Notification::via_email(
                Entity::Node("delivery-node.zzz".into()),
                "abuse@regthree.zzz".into()
            ),
            Notification::via_email(
                Entity::Node("10.10.10.10".into()),
                "abuse@providerone.zzz".into()
            )
        ]
    }

    fn parsed_mail() -> ParsedMail {
        ParsedMail {
            authentication_results: None,
            delivery_nodes: vec![],
            email_addresses: email_addresses(),
            fulfillment_nodes: vec![],
            subject: None,
        }
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![],
            links: vec![],
            reply_to: vec![],
            return_path: vec![]
        }
    }

    fn build_delivery_nodes() -> Vec<DeliveryNode> {
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

    fn reportable_email_addresses() -> EmailAddresses {
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
}

#[cfg(test)]
mod build_notifications_from_email_addresses_tests {
    use crate::data::{
        Domain,
        EmailAddresses,
        EmailAddressData,
        Registrar,
    };
    use super::*;

    #[test]
    fn generates_email_address_notifications_for_each_type_of_address() {
       let addresses = email_addresses();

       assert_eq!(
           expected_notifications(),
           build_notifications_from_email_addresses(&addresses)
       );
    }

    #[test]
    fn discards_any_cases_where_notification_could_not_be_generated() {
       let addresses = email_addresses_with_unnotifiable_entries();

       assert_eq!(
           expected_notifications(),
           build_notifications_from_email_addresses(&addresses)
       );
    }

    fn email_addresses() -> EmailAddresses {
        EmailAddresses {
            from: vec![
                email_address_data("from_1@test.com", "abuse@regone.zzz"),
                email_address_data("from_2@test.com", "abuse@regtwo.zzz"),
            ],
            links: vec![
                email_address_data("link_1@test.com", "abuse@regthree.zzz"),
                email_address_data("link_2@test.com", "abuse@regfour.zzz"),
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
                name: None,
            }),
        }
    }

    fn expected_notifications() -> Vec<Notification> {
        vec![
            Notification::via_email(
                Entity::EmailAddress("from_1@test.com".into()),
                "abuse@regone.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("from_2@test.com".into()),
                "abuse@regtwo.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("link_1@test.com".into()),
                "abuse@regthree.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("link_2@test.com".into()),
                "abuse@regfour.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("reply_to_1@test.com".into()),
                "abuse@regfive.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("reply_to_2@test.com".into()),
                "abuse@regsix.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("return_path_1@test.com".into()),
                "abuse@regseven.zzz".into()
            ),
            Notification::via_email(
                Entity::EmailAddress("return_path_2@test.com".into()),
                "abuse@regeight.zzz".into()
            ),
        ]
    }

    fn email_addresses_with_unnotifiable_entries() -> EmailAddresses {
        EmailAddresses {
            from: vec![
                email_address_data("from_1@test.com", "abuse@regone.zzz"),
                email_address_data("from_2@test.com", "abuse@regtwo.zzz"),
                email_address_data_no_registrar("from_3@test.com"),
            ],
            links: vec![
                email_address_data("link_1@test.com", "abuse@regthree.zzz"),
                email_address_data("link_2@test.com", "abuse@regfour.zzz"),
                email_address_data_no_registrar("link_3@test.com"),
            ],
            reply_to: vec![
                email_address_data("reply_to_1@test.com", "abuse@regfive.zzz"),
                email_address_data("reply_to_2@test.com", "abuse@regsix.zzz"),
                email_address_data_no_registrar("reply_to_3@test.com"),
            ],
            return_path: vec![
                email_address_data("return_path_1@test.com", "abuse@regseven.zzz"),
                email_address_data("return_path_2@test.com", "abuse@regeight.zzz"),
                email_address_data_no_registrar("return_path_3@test.com"),
            ]
        }
    }

    fn email_address_data_no_registrar(address: &str) -> EmailAddressData {
        EmailAddressData {
            address: address.into(),
            domain: Domain::from_email_address(address),
            registrar: None
        }
    }
}

#[cfg(test)]
mod build_notification_from_email_address_tests {
    use crate::data::{Domain, DomainCategory, Registrar};
    use super::*;

    #[test]
    fn builds_notification_using_registrar_data_if_no_domain_abuse_address() {
        let data = email_address_data_no_domain_abuse_address();

        let expected_response = Some(
            Notification::via_email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@regone.zzz".into()
            )
        );

        assert_eq!(expected_response, build_notification_from_email_address(&data));
    }

    #[test]
    fn builds_notification_using_registrar_data_if_no_domain() {
        let data = email_address_data_no_domain();

        let expected_response = Some(
            Notification::via_email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@regone.zzz".into()
            )
        );

        assert_eq!(expected_response, build_notification_from_email_address(&data));
    }

    #[test]
    fn prefers_to_notify_using_domain_abuse_address() {
        let data = email_address_data_with_domain_abuse_address();

        let expected_response = Some(
            Notification::via_email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@test.com".into()
            )
        );

        assert_eq!(expected_response, build_notification_from_email_address(&data));
    }

    #[test]
    fn returns_none_if_registrar_can_not_be_notified() {
        let data = email_address_data_no_registrar();

        assert_eq!(None, build_notification_from_email_address(&data));
    }

    fn email_address_data_no_domain_abuse_address() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: Domain::from_email_address("foo@test.com"),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn email_address_data_no_domain() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn email_address_data_with_domain_abuse_address() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: Some(domain_with_abuse_address()),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn domain_with_abuse_address() -> Domain {
        Domain {
            abuse_email_address: Some("abuse@test.com".into()),
            category: DomainCategory::OpenEmailProvider,
            name: "test.com".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }

    fn email_address_data_no_registrar() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: Domain::from_email_address("foo@test.com"),
            registrar: None,
        }
    }
}

#[cfg(test)]
mod build_notifications_from_fufillment_nodes_tests {
    use crate::data::{Node, Registrar};
    use super::*;

    #[test]
    fn builds_notifications_for_each_node() {
        assert_eq!(
            expected(),
            build_notifications_from_fulfillment_nodes(&input())
        )
    }

    #[test]
    fn excludes_nodes_that_cannot_have_notifications() {
        assert_eq!(
            expected(),
            build_notifications_from_fulfillment_nodes(&input_with_nodes_that_cannot_be_notified())
        )
    }

    fn input() -> Vec<FulfillmentNode> {
        vec![
            fulfillment_node("https://dodgy.phishing.link", "abuse@regone.zzz"),
            fulfillment_node("https://also.dodgy.phishing.link", "abuse@regtwo.zzz"),
        ]
    }

    fn input_with_nodes_that_cannot_be_notified() -> Vec<FulfillmentNode> {
        vec![
            fulfillment_node("https://dodgy.phishing.link", "abuse@regone.zzz"),
            fulfillment_node_that_cannot_be_notified("https://nonotify.phishing.link"),
            fulfillment_node("https://also.dodgy.phishing.link", "abuse@regtwo.zzz"),
        ]
    }

    fn expected() -> Vec<Notification> {
        vec![
            Notification::via_email(
                Entity::Node("https://dodgy.phishing.link".into()), "abuse@regone.zzz".into()
            ),
            Notification::via_email(
                Entity::Node("https://also.dodgy.phishing.link".into()), "abuse@regtwo.zzz".into()
            ),
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

    fn fulfillment_node_that_cannot_be_notified(url: &str) -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            investigable: true,
            visible: Node {
                domain: None,
                registrar: None,
                url: url.into(),
            },
        }
    }
}

#[cfg(test)]
mod build_notifications_from_fulfillment_node_tests {
    use crate::data::{Node, Registrar};
    use super::*;

    #[test]
    fn hidden_and_visible_builds_notifications_for_both() {
        let f_node = fulfillment_node_both();
        let expected = vec![
            Notification::via_email(
                Entity::Node("https://hidden.phishing.link".into()), "abuse@reghidden.zzz".into()
            ),
            Notification::via_email(
                Entity::Node("https://visible.phishing.link".into()), "abuse@regvisible.zzz".into()
            ),
        ];

        assert_eq!(expected, build_notifications_from_fulfillment_node(&f_node));
    }

    #[test]
    fn visible_only_builds_notifications_for_visible() {
        let f_node = fulfillment_node_visible_only();
        let expected = vec![
            Notification::via_email(
                Entity::Node("https://visible.phishing.link".into()), "abuse@regvisible.zzz".into()
            ),
        ];

        assert_eq!(expected, build_notifications_from_fulfillment_node(&f_node));
    }

    #[test]
    fn no_notifiable_nodes_returns_empty_collection() {
        let f_node = fulfillment_node_no_notifiable_nodes();
        let expected: Vec<Notification> = vec![];

        assert_eq!(expected, build_notifications_from_fulfillment_node(&f_node))
    }

    fn fulfillment_node_both() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@reghidden.zzz".into()),
                    name: None,
                }),
                url: "https://hidden.phishing.link".into(),
            }),
            investigable: true,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regvisible.zzz".into()),
                    name: None,
                }),
                url: "https://visible.phishing.link".into(),
            },
        }
    }

    fn fulfillment_node_visible_only() -> FulfillmentNode {
        FulfillmentNode {
            hidden: None,
            investigable: true,
            visible: Node {
                domain: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@regvisible.zzz".into()),
                    name: None,
                }),
                url: "https://visible.phishing.link".into(),
            },
        }
    }

    fn fulfillment_node_no_notifiable_nodes() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(Node {
                domain: None,
                registrar: None,
                url: "https://hidden.phishing.link".into(),
            }),
            investigable: true,
            visible: Node {
                domain: None,
                registrar: None,
                url: "https://visible.phishing.link".into(),
            },
        }
    }
}

#[cfg(test)]
mod build_notifications_from_delivery_nodes_tests {
    use crate::data::{DeliveryNode, Domain, DomainCategory, HostNode};

    use super::*;

    #[test]
    fn builds_notifications_for_delivery_nodes() {
        assert_eq!(
            expected(), build_notifications_from_delivery_nodes(&input())
        );
    }

    fn input() -> Vec<DeliveryNode> {
        vec![
            delivery_node("node1.zzz", "abuse@regone.zzz"),
            delivery_node("node2.zzz", "abuse@regtwo.zzz"),
        ]
    }

    fn delivery_node(name: &str, abuse_email_address: &str) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: name.into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: None,
                ip_address: None,
                registrar: Some(Registrar {
                    abuse_email_address: Some(abuse_email_address.into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn expected() -> Vec<Notification> {
        vec![
            Notification::via_email(Entity::Node("node1.zzz".into()), "abuse@regone.zzz".into()),
            Notification::via_email(Entity::Node("node2.zzz".into()), "abuse@regtwo.zzz".into()),
        ]
    }
}

#[cfg(test)]
mod build_notifications_for_delivery_node_tests {
    use crate::data::{Domain, DomainCategory, InfrastructureProvider};
    use super::*;

    #[test]
    fn returns_notifications_for_delivery_node() {
        assert_eq!(
            vec![
                Notification::via_email(Entity::Node("phishing.zzz".into()), "abuse@regone.zzz".into()),
                Notification::via_email(
                    Entity::Node("10.10.10.10".into()),
                    "abuse@providerone.zzz".into()
                ),
            ],
            build_notifications_for_delivery_node(&delivery_node())
        );
    }

    #[test]
    fn excludes_notifications_that_result_in_none() {
        let expected: Vec<Notification> = vec![];

        assert_eq!(
            expected,
            build_notifications_for_delivery_node(&delivery_node_no_provider_no_registrar())
        );
    }

    #[test]
    fn returns_empty_collection_if_no_observed_sender() {
        let expected: Vec<Notification> = vec![];

        assert_eq!(
            expected,
            build_notifications_for_delivery_node(&delivery_node_no_observed_sender())
        );
    }

    fn delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: Some(HostNode{
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "fake.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@fakeproviderone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("100.100.100.100".into()),
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@fakeregone.zzz".into()),
                    name: None,
                }),
            }),
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "phishing.zzz".into(),
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
                    abuse_email_address: Some("abuse@regone.zzz".into()),
                    name: None,
                }),
            }),
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }

    fn delivery_node_no_provider_no_registrar() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: Some(HostNode{
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "fake.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@fakeproviderone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("100.100.100.100".into()),
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@fakeregone.zzz".into()),
                    name: None,
                }),
            }),
            observed_sender: Some(HostNode {
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "phishing.zzz".into(),
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

    fn delivery_node_no_observed_sender() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: Some(HostNode{
                domain: Some(Domain {
                    abuse_email_address: None,
                    category: DomainCategory::Other,
                    name: "fake.zzz".into(),
                    registration_date: None,
                    resolved_domain: None,
                }),
                host: None,
                infrastructure_provider: Some(InfrastructureProvider {
                    abuse_email_address: Some("abuse@fakeproviderone.zzz".into()),
                    name: None,
                }),
                ip_address: Some("100.100.100.100".into()),
                registrar: Some(Registrar {
                    abuse_email_address: Some("abuse@fakeregone.zzz".into()),
                    name: None,
                }),
            }),
            observed_sender: None,
            position: 0,
            recipient: None,
            time: None,
            trusted: true,
        }
    }
}

#[cfg(test)]
mod build_notification_for_delivery_node_domain_tests {
    use crate::data::{Domain, DomainCategory};
    use super::*;

    #[test]
    fn returns_notification_for_domain() {
        let notification = Notification::via_email(
            Entity::Node("phishing.zzz".into()),
            "abuse@regone.zzz".into()
        );
        assert_eq!(
            Some(notification),
            build_notification_for_delivery_node_domain(&host_node())
        );
    }

    #[test]
    fn returns_none_if_registrar_cannot_be_notified() {
        assert!(build_notification_for_delivery_node_domain(&host_node_sans_registrar()).is_none())
    }

    #[test]
    fn returns_none_if_no_domain() {
        assert!(build_notification_for_delivery_node_domain(&host_node_sans_domain()).is_none())
    }

    fn host_node() -> HostNode {
        HostNode {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "phishing.zzz".into(),
                registration_date: None,
                resolved_domain: None,
            }),
            host: None,
            infrastructure_provider: None,
            ip_address: None,
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn host_node_sans_domain() -> HostNode {
        HostNode {
            domain: None,
            host: None,
            infrastructure_provider: None,
            ip_address: None,
            registrar: None,
        }
    }

    fn host_node_sans_registrar() -> HostNode {
        HostNode {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: "phishing.zzz".into(),
                registration_date: None,
                resolved_domain: None,
            }),
            host: None,
            infrastructure_provider: None,
            ip_address: None,
            registrar: None
        }
    }
}

#[cfg(test)]
mod build_notification_for_delivery_node_ip_tests {
    use crate::data::InfrastructureProvider;
    use super::*;

    #[test]
    fn returns_notification_for_ip_address() {
        let notification = Notification::via_email(
            Entity::Node("10.10.10.10".into()), "abuse@providerone.zzz".into()
        );

        assert_eq!(
            Some(notification),
            build_notification_for_delivery_node_ip(&host_node())
        );
    }

    #[test]
    fn returns_none_if_no_infrastructure_provider() {
        assert!(
            build_notification_for_delivery_node_ip(
                &host_node_sans_infrastructure_provider()
            ).is_none()
        );
    }

    #[test]
    fn returns_none_if_no_abuse_email_address() {
        assert!(
            build_notification_for_delivery_node_ip(
                &host_node_sans_abuse_address()
            ).is_none()
        );
    }

    #[test]
    fn returns_none_if_no_ip_address() {
        assert!(
            build_notification_for_delivery_node_ip(
                &host_node_sans_ip_address()
            ).is_none()
        );
    }

    fn host_node() -> HostNode {
        HostNode {
            domain: None,
            host: None,
            infrastructure_provider: Some(InfrastructureProvider {
                abuse_email_address: Some("abuse@providerone.zzz".into()),
                name: None,
            }),
            ip_address: Some("10.10.10.10".into()),
            registrar: None,
        }
    }

    fn host_node_sans_ip_address() -> HostNode {
        HostNode {
            domain: None,
            host: None,
            infrastructure_provider: Some(InfrastructureProvider {
                abuse_email_address: Some("abuse@providerone.zzz".into()),
                name: None,
            }),
            ip_address: None,
            registrar: None,
        }
    }

    fn host_node_sans_infrastructure_provider() -> HostNode {
        HostNode {
            domain: None,
            host: None,
            infrastructure_provider: None,
            ip_address: Some("10.10.10.10".into()),
            registrar: None,
        }
    }

    fn host_node_sans_abuse_address() -> HostNode {
        HostNode {
            domain: None,
            host: None,
            infrastructure_provider: Some(InfrastructureProvider {
                abuse_email_address: None,
                name: None,
            }),
            ip_address: Some("10.10.10.10".into()),
            registrar: None,
        }
    }
}

#[cfg(test)]
mod build_notification_for_registrar_tests {
    use super::*;

    #[test]
    fn returns_notification_if_registrar_has_abuse_email_address() {
        let registrar = build_registrar(Some("abuse@registrar.zzz"));
        let notification = Notification::via_email(entity(), "abuse@registrar.zzz".into());

        assert_eq!(
            Some(notification),
            build_notification_for_registrar(Some(&registrar), entity())
        );
    }

    #[test]
    fn returns_none_if_registrar_has_no_abuse_email_address() {
        let registrar = build_registrar(None);

        assert!(build_notification_for_registrar(Some(&registrar), entity()).is_none());
    }

    #[test]
    fn returns_none_if_no_registrar() {
        assert!(build_notification_for_registrar(None, entity()).is_none());
    }

    fn build_registrar(abuse_email_address: Option<&str>) -> Registrar {
        Registrar {
            abuse_email_address: abuse_email_address.map(|v| v.into()),
            name: None,
        }
    }

    fn entity() -> Entity {
        Entity::Node("https://phishing.link".into())
    }
}

#[cfg(test)]
mod build_notification_for_domain_tests {
    use crate::data::DomainCategory;

    use super::*;

    #[test]
    fn returns_notification_for_domain() {
        let domain = domain_with_abuse_address();

        assert_eq!(
            Some(Notification::via_email(entity(), "abuse@test.com".into())),
            build_notification_for_domain(&domain, entity())
        );
    }

    #[test]
    fn returns_none_if_domain_has_no_email_address() {
        let domain = domain_without_email_address();
        let notification = build_notification_for_domain(&domain, entity());

        assert!(notification.is_none());
    }

    fn entity() -> Entity {
        Entity::EmailAddress("foo@test.com".into())
    }

    fn domain_with_abuse_address() -> Domain {
        Domain {
            abuse_email_address: Some("abuse@test.com".into()),
            category: DomainCategory::OpenEmailProvider,
            name: "test.com".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }

    fn domain_without_email_address() -> Domain {
        Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "test.com".into(),
            registration_date: None,
            resolved_domain: None,
        }
    }
}

#[cfg(test)]
mod build_notification_for_node_tests {
    use crate::data::DomainCategory;
    use super::*;

    #[test]
    fn notifies_on_domain_abuse_address_if_both_node_and_domain() {
        let node = node_with_domain_and_registrar();
        let expected = Some(Notification::via_email(
            Entity::Node( "https://hidden.phishing.link".into()),
            "abuse@domainowner.zzz".into()
        ));

        assert_eq!(expected, build_notification_for_node(Some(&node)));
    }

    #[test]
    fn notifies_on_registrar_abuse_address_if_no_domain_notification() {
        let node = node_with_no_notifiable_domain();
        let expected = Some(Notification::via_email(
            Entity::Node( "https://hidden.phishing.link".into()),
            "abuse@reghidden.zzz".into()
        ));

        assert_eq!(expected, build_notification_for_node(Some(&node)));
    }

    #[test]
    fn notifies_on_registrar_abuse_address_if_no_domain() {
        let node = node_with_no_domain();
        let expected = Some(Notification::via_email(
            Entity::Node( "https://hidden.phishing.link".into()),
            "abuse@reghidden.zzz".into()
        ));

        assert_eq!(expected, build_notification_for_node(Some(&node)));
    }

    #[test]
    fn does_not_notify_if_no_domain_and_no_notifiable_registrar() {
        let node = node_with_no_domain_no_notifiable_registrar();

        assert_eq!(None, build_notification_for_node(Some(&node)));
    }

    #[test]
    fn does_not_notify_if_no_notifiable_domain_and_no_notifiable_registrar() {
        let node = node_with_no_notifiable_domain_no_notifiable_registrar();

        assert_eq!(None, build_notification_for_node(Some(&node)));
    }

    #[test]
    fn does_not_notify_if_no_node() {
        assert_eq!(None, build_notification_for_node(None));
    }

    fn node_with_domain_and_registrar() -> Node {
        Node {
            domain: Some(Domain {
                abuse_email_address: Some("abuse@domainowner.zzz".into()),
                category: DomainCategory::UrlShortener,
                name: "phishing.link".into(),
                registration_date: None,
                resolved_domain: None
            }),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@reghidden.zzz".into()),
                name: None,
            }),
            url: "https://hidden.phishing.link".into(),
        }
    }

    fn node_with_no_notifiable_domain() -> Node {
        Node {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::UrlShortener,
                name: "phishing.link".into(),
                registration_date: None,
                resolved_domain: None
            }),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@reghidden.zzz".into()),
                name: None,
            }),
            url: "https://hidden.phishing.link".into(),
        }
    }

    fn node_with_no_domain() -> Node {
        Node {
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@reghidden.zzz".into()),
                name: None,
            }),
            url: "https://hidden.phishing.link".into(),
        }
    }

    fn node_with_no_domain_no_notifiable_registrar() -> Node {
        Node {
            domain: None,
            registrar: Some(Registrar {
                abuse_email_address: None,
                name: None,
            }),
            url: "https://hidden.phishing.link".into(),
        }
    }

    fn node_with_no_notifiable_domain_no_notifiable_registrar() -> Node {
        Node {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::UrlShortener,
                name: "phishing.link".into(),
                registration_date: None,
                resolved_domain: None
            }),
            registrar: Some(Registrar {
                abuse_email_address: None,
                name: None,
            }),
            url: "https://hidden.phishing.link".into(),
        }
    }
}
