use crate::data::{EmailAddressData, EmailAddresses, OutputData};
use crate::mailer::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Notification {
    Email(Entity, String)
}

pub fn add_notifications(data: OutputData) -> OutputData {
    let entities = data.reportable_entities.clone().unwrap();

    let notifications = vec![
        build_notifications_from_email_addresses(&entities.email_addresses),
        // build_notifications_from_fulfillment_nodes(
        //     &entities.fulfillment_nodes_container.nodes
        // ),
        // build_notifications_from_delivery_nodes(&entities.delivery_nodes),
    ]
    .into_iter()
    .flatten()
    .collect();

    OutputData {
        notifications,
        ..data
    }
}

#[cfg(test)]
mod add_notification_test {
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
    fn adds_a_notification_for_a_reportable_email() {
       let data = build_output_data();

       let data_with_notifications = add_notifications(data);

       assert_eq!(
           expected_output_data(build_output_data()),
           data_with_notifications
       );
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

    fn expected_output_data(input_data: OutputData) -> OutputData {
        OutputData {
            notifications: expected_notifications(),
            ..input_data
        }
    }

    fn expected_notifications() -> Vec<Notification> {
        vec![
            Notification::Email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@regone.zzz".into()
            ),
            Notification::Email(
                Entity::Node("https://dodgy.phishing.link".into()),
                "abuse@regtwo.zzz".into()
            ),
            Notification::Email(
                Entity::Node("delivery-node.zzz".into()),
                "abuse@regthree.zzz".into()
            ),
            Notification::Email(
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

fn build_notifications_from_email_addresses(addresses: &EmailAddresses) -> Vec<Notification> {
    [
        to_refs(&addresses.from),
        to_refs(&addresses.links),
        to_refs(&addresses.reply_to),
        to_refs(&addresses.return_path)
    ]
    .iter()
    .flatten()
    .map(|address_data| build_notification_from_email_address(address_data).unwrap())
    .collect()
    // .iter()
    // .flatten()
    // .map(|address_data| build_notification_from_email_address(&address_data).unwrap())
    // .collect()
}

fn to_refs(data: &[EmailAddressData]) -> Vec<&EmailAddressData> {
    data.iter().collect()
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
            Notification::Email(
                Entity::EmailAddress("from_1@test.com".into()),
                "abuse@regone.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("from_2@test.com".into()),
                "abuse@regtwo.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("link_1@test.com".into()),
                "abuse@regthree.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("link_2@test.com".into()),
                "abuse@regfour.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("reply_to_1@test.com".into()),
                "abuse@regfive.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("reply_to_2@test.com".into()),
                "abuse@regsix.zzz".into()
            ),
            Notification::Email(
                Entity::EmailAddress("return_path_1@test.com".into()),
                "abuse@regseven.zzz".into()
            ),
            Notification::Email(
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

fn build_notification_from_email_address(data: &EmailAddressData) -> Option<Notification> {
    if let Some(registrar) = &data.registrar {
        registrar.abuse_email_address.as_ref().map(|abuse_email_address| {
            Notification::Email(
                Entity::EmailAddress(data.address.clone()),
                abuse_email_address.into()
            )
        })
    } else {
        None
    }
}

#[cfg(test)]
mod build_notification_from_email_address_tests {
    use crate::data::{Domain, Registrar};
    use super::*;

    #[test]
    fn builds_notification_from_email_address_data() {
        let data = email_address_data();

        let expected_response = Some(
            Notification::Email(
                Entity::EmailAddress("foo@test.com".into()),
                "abuse@regone.zzz".into()
            )
        );

        assert_eq!(expected_response, build_notification_from_email_address(&data));
    }

    #[test]
    fn registrar_does_not_have_abuse_email_address() {
        let data = email_address_data_no_abuse_address();

        assert_eq!(None, build_notification_from_email_address(&data));
    }

    #[test]
    fn email_address_data_does_not_have_registrar() {
        let data = email_address_data_no_registrar();

        assert_eq!(None, build_notification_from_email_address(&data));
    }

    fn email_address_data() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: Domain::from_email_address("foo@test.com"),
            registrar: Some(Registrar {
                abuse_email_address: Some("abuse@regone.zzz".into()),
                name: None,
            }),
        }
    }

    fn email_address_data_no_abuse_address() -> EmailAddressData {
        EmailAddressData {
            address: "foo@test.com".into(),
            domain: Domain::from_email_address("foo@test.com"),
            registrar: Some(Registrar {
                abuse_email_address: None,
                name: None,
            }),
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
