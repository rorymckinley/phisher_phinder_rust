use chrono::prelude::*;
use crate::data::{
    DeliveryNode,
    Domain,
    DomainCategory,
    EmailAddressData,
    FulfillmentNode,
    HostNode,
    InfrastructureProvider,
    Node,
    OutputData,
    ParsedMail,
    Registrar,
    EmailAddresses
};
use std::str::FromStr;
use test_friendly_rdap_client::bootstrap::Bootstrap;
use test_friendly_rdap_client::Client;
use test_friendly_rdap_client::parser;
use std::sync::Arc;

#[cfg(test)]
mod populate_tests {
    use super::*;
    use crate:: data::{
        Domain,
        EmailAddressData,
        InfrastructureProvider,
        ParsedMail,
        Registrar,
        EmailAddresses
    };
    use crate::mountebank::*;

    #[test]
    fn populates_from_email_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            vec! [
                EmailAddressData {
                    address: "someone@fake.net".into(),
                    domain: domain_object(
                        "fake.net",
                        Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                    ),
                    registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                }
            ],
            actual.parsed_mail.email_addresses.from
        );
    }

    #[test]
    fn populates_reply_to_email_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            vec! [
                EmailAddressData {
                    address: "anyone@possiblynotfake.com".into(),
                    domain: domain_object(
                        "possiblynotfake.com",
                        Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()),
                    ),
                    registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
                },
            ],
            actual.parsed_mail.email_addresses.reply_to
        );
    }

    #[test]
    fn populates_return_path_email_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            vec! [
                EmailAddressData {
                    address: "everyone@morethanlikelyfake.net".into(),
                    domain: domain_object(
                        "morethanlikelyfake.net",
                        Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                    ),
                    registrar: registrar_object("Reg Three", Some("abuse@regthree.zzz")),
                },
            ],
            actual.parsed_mail.email_addresses.return_path
        );
    }

    #[test]
    fn populates_link_email_addresses() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            vec! [
                EmailAddressData {
                    address: "perp@alsofake.net".into(),
                    domain: domain_object(
                        "alsofake.net",
                        Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 15).unwrap()),
                    ),
                    registrar: registrar_object("Reg Four", Some("abuse@regfour.zzz")),
                }
            ],
            actual.parsed_mail.email_addresses.links
        );
    }

    #[test]
    fn update_fulfillment_nodes() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            vec![
                FulfillmentNode {
                    hidden: None,
                    visible: Node {
                        domain: domain_object(
                            "iamascamsite.com",
                            Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 16).unwrap())
                        ),
                        registrar: registrar_object("Reg Five", Some("abuse@regfive.zzz")),
                        url: "https://iamascamsite.com".into(),
                    }
                }
            ],
            actual.parsed_mail.fulfillment_nodes
        );
    }

    #[test]
    fn updates_delivery_nodes() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(
            sorted(vec![
                output_delivery_node(
                    "host.dodgyaf.com",
                    "Reg Six",
                    "abuse@regsix.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap(),
                    "10.10.10.10",
                    "Acme Hosting",
                    "abuse@acmehost.zzz",
                    30,
                ),
                output_delivery_node(
                    "host.probablyalsoascammer.net",
                    "Reg Seven",
                    "abuse@regseven.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 18).unwrap(),
                    "192.168.10.10",
                    "Hackme Hosting",
                    "abuse@hackmehost.zzz",
                    31,
                )
            ]),
            sorted(actual.parsed_mail.delivery_nodes)
        );
    }

    fn input_data() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                delivery_nodes: vec![
                    input_delivery_node("host.dodgyaf.com", "10.10.10.10", 30),
                    input_delivery_node("host.probablyalsoascammer.net", "192.168.10.10", 31)
                ],
                fulfillment_nodes: vec![
                    FulfillmentNode::new("https://iamascamsite.com"),
                ],
                subject: Some("Does not matter".into()),
                email_addresses: EmailAddresses {
                    from: vec![
                        EmailAddressData {
                            address: "someone@fake.net".into(),
                            domain: domain_object("fake.net", None),
                            registrar: None,
                        }
                    ],
                    links: vec![
                        EmailAddressData {
                            address: "perp@alsofake.net".into(),
                            domain: domain_object("alsofake.net", None),
                            registrar: None,
                        }
                    ],
                    reply_to: vec![
                        EmailAddressData {
                            address: "anyone@possiblynotfake.com".into(),
                            domain: domain_object("possiblynotfake.com", None),
                            registrar: None,
                        },
                    ],
                    return_path: vec![
                        EmailAddressData {
                            address: "everyone@morethanlikelyfake.net".into(),
                            domain: domain_object("morethanlikelyfake.net", None),
                            registrar: None,
                        },
                    ],
                },
            },
            raw_mail: "raw mail text goes here".into()
        }
    }

    fn input_delivery_node(host_name: &str, ip_address: &str, seconds: u32) -> DeliveryNode {
        let time = Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, seconds).unwrap());

        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode::new(Some(host_name), Some(ip_address)).unwrap()),
            recipient: None,
            time,
        }
    }

    fn output_delivery_node(
        host_name: &str,
        registrar_name: &str,
        registrar_abuse_email_address: &str,
        registration_date: DateTime<Utc>,
        ip_address: &str,
        infrastructure_provider_name: &str,
        infrastructure_provider_email_address: &str,
        seconds: u32,
    ) -> DeliveryNode {
        let time = Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, seconds).unwrap());

        let observed_sender = Some(
            HostNode {
                domain: domain_object(host_name, Some(registration_date)),
                host: Some(host_name.into()),
                infrastructure_provider: infrastructure_provider_instance(
                    infrastructure_provider_name, Some(infrastructure_provider_email_address)
                ),
                ip_address: Some(ip_address.into()),
                registrar: registrar_object(registrar_name, Some(registrar_abuse_email_address)),
            }
        );

        DeliveryNode {
            advertised_sender: None,
            observed_sender,
            recipient: None,
            time,
        }
    }

    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn infrastructure_provider_instance(
        name: &str, abuse_email_address: Option<&str>
    ) -> Option<InfrastructureProvider> {
        Some(
            InfrastructureProvider {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    None,
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    None,
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
                DnsServerConfig::response_200(
                    "morethanlikelyfake.net",
                    None,
                    "Reg Three",
                    "abuse@regthree.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()
                ),
                DnsServerConfig::response_200(
                    "alsofake.net",
                    None,
                    "Reg Four",
                    "abuse@regfour.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 15).unwrap()
                ),
                DnsServerConfig::response_200(
                    "iamascamsite.com",
                    None,
                    "Reg Five",
                    "abuse@regfive.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 16).unwrap()
                ),
                DnsServerConfig::response_200(
                    "host.dodgyaf.com",
                    None,
                    "Reg Six",
                    "abuse@regsix.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                ),
                DnsServerConfig::response_200(
                    "host.probablyalsoascammer.net",
                    None,
                    "Reg Seven",
                    "abuse@regseven.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 18).unwrap()
                ),
            ]
        );

        setup_ip_v4_server(vec![
            IpServerConfig::response_200(
                "10.10.10.10",
                None,
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")])
            ),
            IpServerConfig::response_200(
                "192.168.10.10",
                None,
                ("192.168.0.0", "192.168.255.255"),
                Some(&[("Hackme Hosting", "registrant", "abuse@hackmehost.zzz")])
            )
        ]);
    }

    async fn get_bootstrap() -> Bootstrap {
        let mut client = Client::new();

        client.set_base_bootstrap_url("http://localhost:4545");

        client.fetch_bootstrap().await.unwrap()
    }

    fn sorted(mut nodes: Vec<DeliveryNode>) -> Vec<DeliveryNode> {
        nodes.sort_by_key(|node| node.time );
        nodes
    }
}

pub async fn populate(bootstrap: Bootstrap, data: OutputData) -> OutputData {
    let b_strap = Arc::new(bootstrap);
    let update_from = lookup_email_address_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.email_addresses.from
    );
    let update_links = lookup_email_address_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.email_addresses.links
    );
    let update_reply_to = lookup_email_address_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.email_addresses.reply_to
    );
    let update_return_path = lookup_email_address_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.email_addresses.return_path
    );
    let update_fulfillment_nodes = lookup_fulfillment_nodes_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.fulfillment_nodes
    );
    let update_delivery_nodes = lookup_delivery_nodes_from_rdap(
        Arc::clone(&b_strap),
        data.parsed_mail.delivery_nodes
    );

    let (from, links, reply_to, return_path, fulfillment_nodes, delivery_nodes) = tokio::join!(
        update_from,
        update_links,
        update_reply_to,
        update_return_path,
        update_fulfillment_nodes,
        update_delivery_nodes,
    );

    let email_addresses = EmailAddresses {
        from,
        reply_to,
        return_path,
        links,
    };

    OutputData {
        parsed_mail: ParsedMail {
            email_addresses,
            fulfillment_nodes,
            delivery_nodes,
            ..data.parsed_mail
        },
        ..data
    }
}

#[cfg(test)]
mod lookup_email_address_from_rdap_tests {
    use super::*;
    use crate::mountebank::*;
    use crate:: data::Registrar;
    use test_support::*;

    #[test]
    fn populates_email_address_data_with_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input();

        let actual = tokio_test::block_on(
            lookup_email_address_from_rdap(Arc::new(bootstrap), input)
        );

        assert_eq!(sorted(populated_output()), sorted(actual));
    }

    fn input() -> Vec<EmailAddressData> {
        vec![
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: domain_object("fake.net", None, DomainCategory::Other),
                registrar: None,
            },
            EmailAddressData {
                address: "anyone@possiblynotfake.com".into(),
                domain: domain_object("possiblynotfake.com", None, DomainCategory::Other),
                registrar: None,
            },
        ]
    }

    fn populated_output() -> Vec<EmailAddressData> {
        vec![
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: domain_object(
                    "fake.net",
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                    DomainCategory::Other,
                ),
                registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
            },
            EmailAddressData {
                address: "anyone@possiblynotfake.com".into(),
                domain: domain_object(
                    "possiblynotfake.com",
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()),
                    DomainCategory::Other,
                ),
                registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
            },
        ]
    }

    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
        category: DomainCategory
    ) ->  Option<Domain> {
        Some(
            Domain {
                category,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn sorted(mut addresses: Vec<EmailAddressData>) -> Vec<EmailAddressData> {
        addresses.sort_by(|a,b| a.address.cmp(&b.address));

        addresses
    }
}

async fn lookup_email_address_from_rdap(
    bootstrap: Arc<Bootstrap>, data: Vec<EmailAddressData>
) -> Vec<EmailAddressData> {
    use tokio::task::JoinSet;

    let mut set: JoinSet<EmailAddressData> = JoinSet::new();

    for e_a_d in data.into_iter() {
        let b_strap = Arc::clone(&bootstrap);
        set.spawn(async  move{
            lookup_email_address(b_strap, e_a_d).await
        });
    }

    let mut output = vec![];

    while let Some(res) = set.join_next().await {
        output.push(res.unwrap())
    }

    output
}

#[cfg(test)]
mod lookup_fulfillment_nodes_from_rdap_tests {
    use super::*;
    use crate::mountebank::*;
    use test_support::*;

    #[test]
    fn populates_fulfillment_nodes_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_fulfillment_nodes_from_rdap(Arc::new(bootstrap), input())
        );

        assert_eq!(sorted(output()), sorted(actual));
    }

    fn input() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode::new("https://fake.net"),
            FulfillmentNode::new("https://possiblynotfake.com")
        ]
    }

    fn output() -> Vec<FulfillmentNode> {
        vec![
            FulfillmentNode {
                hidden: None,
                visible: Node {
                    domain: domain_object(
                                "fake.net",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap())
                            ),
                            registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                            url: "https://fake.net".into(),
                }
            },
            FulfillmentNode {
                hidden: None,
                visible: Node {
                    domain: domain_object(
                                "possiblynotfake.com",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap())
                            ),
                            registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
                            url: "https://possiblynotfake.com".into(),
                }
            },
        ]
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    None,
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    None,
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
            ]
        );
    }

    // test_support rather?
    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn sorted(mut nodes: Vec<FulfillmentNode>) -> Vec<FulfillmentNode> {
        nodes.sort_by_key(|node| String::from(node.visible_url()));
        nodes
    }
}

async fn lookup_fulfillment_nodes_from_rdap(
    bootstrap: Arc<Bootstrap>, data: Vec<FulfillmentNode>
) -> Vec<FulfillmentNode> {
    use tokio::task::JoinSet;

    let mut set: JoinSet<FulfillmentNode> = JoinSet::new();

    for node in data.into_iter() {
        let b_strap = Arc::clone(&bootstrap);
        set.spawn(async  move{
            lookup_fulfillment_node(b_strap, node).await
        });
    }

    let mut output = vec![];

    while let Some(res) = set.join_next().await {
        output.push(res.unwrap())
    }

    output
}

#[cfg(test)]
mod lookup_delivery_nodes_from_rdap_tests {
    use test_support::*;
    use super::*;
    use crate::data::{HostNode, InfrastructureProvider};
    use crate::mountebank::*;

    #[test]
    fn populates_delivery_nodes_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_delivery_nodes_from_rdap(Arc::new(bootstrap), input())
        );

        assert_eq!(sorted(output()), sorted(actual));
    }

    fn input() -> Vec<DeliveryNode> {
        vec![
            input_delivery_node("host.dodgyaf.com", "10.10.10.10", 30),
            input_delivery_node("host.probablyalsoascammer.net", "192.168.10.10", 31)
        ]
    }

    fn output() -> Vec<DeliveryNode> {
        vec![
            output_delivery_node(
                "host.dodgyaf.com",
                "Reg Six",
                "abuse@regsix.zzz",
                Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap(),
                "10.10.10.10",
                "Acme Hosting",
                "abuse@acmehost.zzz",
                30,
            ),
            output_delivery_node(
                "host.probablyalsoascammer.net",
                "Reg Seven",
                "abuse@regseven.zzz",
                Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 18).unwrap(),
                "192.168.10.10",
                "Hackme Hosting",
                "abuse@hackmehost.zzz",
                31,
            )
        ]
    }

    fn input_delivery_node(host_name: &str, ip_address: &str, seconds: u32) -> DeliveryNode {
        let time = Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, seconds).unwrap());

        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode::new(Some(host_name), Some(ip_address)).unwrap()),
            recipient: None,
            time,
        }
    }

    fn output_delivery_node(
        host_name: &str,
        registrar_name: &str,
        registrar_abuse_email_address: &str,
        registration_date: DateTime<Utc>,
        ip_address: &str,
        infrastructure_provider_name: &str,
        infrastructure_provider_email_address: &str,
        seconds: u32,
    ) -> DeliveryNode {
        let time = Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, seconds).unwrap());

        let observed_sender = Some(
            HostNode {
                domain: domain_object(host_name, Some(registration_date)),
                host: Some(host_name.into()),
                infrastructure_provider: infrastructure_provider_instance(
                    infrastructure_provider_name, Some(infrastructure_provider_email_address)
                ),
                ip_address: Some(ip_address.into()),
                registrar: registrar_object(registrar_name, Some(registrar_abuse_email_address)),
            }
        );
        DeliveryNode {
            advertised_sender: None,
            observed_sender,
            recipient: None,
            time,
        }
    }

    fn infrastructure_provider_instance(
        name: &str, abuse_email_address: Option<&str>
    ) -> Option<InfrastructureProvider> {
        Some(
            InfrastructureProvider {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "host.dodgyaf.com",
                    None,
                    "Reg Six",
                    "abuse@regsix.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                ),
                DnsServerConfig::response_200(
                    "host.probablyalsoascammer.net",
                    None,
                    "Reg Seven",
                    "abuse@regseven.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 18).unwrap()
                ),
            ]
        );

        setup_ip_v4_server(vec![
            IpServerConfig::response_200(
                "10.10.10.10",
                None,
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")])
            ),
            IpServerConfig::response_200(
                "192.168.10.10",
                None,
                ("192.168.0.0", "192.168.255.255"),
                Some(&[("Hackme Hosting", "registrant", "abuse@hackmehost.zzz")])
            )
        ]);
    }

    // test_support rather?
    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn sorted(mut nodes: Vec<DeliveryNode>) -> Vec<DeliveryNode> {
        nodes.sort_by_key(|node| node.time );
        nodes
    }
}

async fn lookup_delivery_nodes_from_rdap(
    bootstrap: Arc<Bootstrap>, data: Vec<DeliveryNode>
) -> Vec<DeliveryNode> {
    use tokio::task::JoinSet;

    let mut set: JoinSet<DeliveryNode> = JoinSet::new();

    for node in data.into_iter() {
        let b_strap = Arc::clone(&bootstrap);
        set.spawn(async  move{
            lookup_delivery_node(b_strap, node).await
        });
    }

    let mut output = vec![];

    while let Some(res) = set.join_next().await {
        output.push(res.unwrap())
    }

    output
}

#[cfg(test)]
mod lookup_fulfillment_node_tests {
    use super::*;
    use crate::mountebank::*;
    use test_support::*;

    #[test]
    fn updates_both_visible_and_hidden_nodes() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_fulfillment_node(Arc::new(bootstrap), input())
        );

        assert_eq!(output(), actual);
    }

    fn input() -> FulfillmentNode {
        let mut node = FulfillmentNode::new("https://fake.net");
        node.set_hidden("https://possiblynotfake.com");

        node
    }

    fn output() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(Node {
                domain: domain_object(
                            "possiblynotfake.com",
                            Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap())
                        ),
                registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
                url: "https://possiblynotfake.com".into(),
            }),
            visible: Node {
                domain: domain_object(
                            "fake.net",
                            Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap())
                        ),
                registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                url: "https://fake.net".into(),
            }
        }
    }

    // test_support rather?
    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    None,
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    None,
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
            ]
        );
    }
}

async fn lookup_fulfillment_node(
    bootstrap: Arc<Bootstrap>, f_node: FulfillmentNode
) -> FulfillmentNode {
    let (hidden, visible) = tokio::join!(
        lookup_node(Arc::clone(&bootstrap), f_node.hidden),
        lookup_node(Arc::clone(&bootstrap), Some(f_node.visible))
    );

    FulfillmentNode {
        hidden,
        visible: visible.unwrap(),
    }
}

#[cfg(test)]
mod lookup_delivery_node_tests {
    use test_support::*;
    use super::*;
    use crate::mountebank::*;

    #[test]
    fn updates_the_observed_sender_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_delivery_node(
                Arc::new(bootstrap), input_delivery_node()
            )
        );

        assert_eq!(output_delivery_node(), actual);
    }

    fn input_delivery_node() -> DeliveryNode {
        DeliveryNode {
            advertised_sender: None,
            observed_sender: Some(HostNode {
                domain: Domain::from_host("host.dodgyaf.com"),
                host: Some("host.dodgyaf.com".into()),
                infrastructure_provider: infrastructure_provider_instance(),
                ip_address: Some("10.10.10.10".into()),
                registrar: None,
            }),
            recipient: None,
            time: None,
        }
    }

    fn output_delivery_node() -> DeliveryNode {
        let domain = Some(Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "host.dodgyaf.com".into(),
            registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap())
        });

        DeliveryNode {
            observed_sender: Some(HostNode {
                domain,
                host: Some("host.dodgyaf.com".into()),
                infrastructure_provider: infrastructure_provider_instance(),
                ip_address: Some("10.10.10.10".into()),
                registrar: registrar_instance(),
            }),
            ..input_delivery_node()
        }
    }

    fn infrastructure_provider_instance() -> Option<InfrastructureProvider> {
        Some(
            InfrastructureProvider {
                name: Some("Acme Hosting".into()),
                abuse_email_address: Some("abuse@acmehost.zzz".into())
            }
        )
    }

    fn registrar_instance() -> Option<Registrar> {
        Some(
            Registrar {
                name: Some("Reg Six".into()),
                abuse_email_address: Some("abuse@regsix.zzz".into())
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "host.dodgyaf.com",
                    None,
                    "Reg Six",
                    "abuse@regsix.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                ),
            ]
        );

        setup_ip_v4_server(vec![
            IpServerConfig::response_200(
                "10.10.10.10",
                None,
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")])
            ),
        ]);
    }
}

async fn lookup_delivery_node(
    bootstrap: Arc<Bootstrap>, d_node: DeliveryNode
) -> DeliveryNode {
    let (observed_sender,) = tokio::join!(
        lookup_host_node(Arc::clone(&bootstrap), d_node.observed_sender)
    );

    DeliveryNode {
        observed_sender,
        ..d_node
    }
}

#[cfg(test)]
mod lookup_node_tests {
    use super::*;
    use crate::mountebank::*;
    use test_support::*;

    #[test]
    fn updates_node_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_node(Arc::new(bootstrap), node("https://fake.net"))
        );

        assert_eq!(Some(populated_node()), actual);
    }

    #[test]
    fn returns_none_if_no_node_provided() {
        clear_all_impostors();
        setup_bootstrap_server();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(lookup_node(Arc::new(bootstrap), None));

        assert_eq!(None, actual);
    }

    #[test]
    fn returns_node_unpopulated_if_no_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(lookup_node(Arc::new(bootstrap), node_sans_domain()));

        assert_eq!(node_sans_domain(), actual);
    }

    #[test]
    fn returns_node_unpopulated_if_no_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(
            lookup_node(Arc::new(bootstrap), node("https://fake.zzz"))
        );

        assert_eq!(node("https://fake.zzz"), actual);
    }

    fn node(url: &str) -> Option<Node> {
        Some(Node::new(url))
    }

    fn node_sans_domain() -> Option<Node> {
        Some(
            Node {
                domain: None,
                registrar: None,
                url: "https://fake.net".into()
            }
        )
    }

    fn populated_node() -> Node {
        Node {
            domain: domain_object(
                        "fake.net", Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap())
                    ),
            registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
            url: "https://fake.net".into(),
        }
    }

    // test_support rather?
    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    None,
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
            ]
        );
    }
}

async fn lookup_node(bootstrap: Arc<Bootstrap>, node_option: Option<Node>) -> Option<Node> {
    if let Some(node) = node_option {
        if let Some(domain) = node.domain {
            if let Some(response) = get_rdap_data(bootstrap, Some(domain.name.clone())).await {
                Some(
                    Node {
                        domain: Some(Domain {
                            registration_date: extract_registration_date(&response.events),
                            ..domain
                        }),
                        registrar: Some(Registrar {
                            abuse_email_address: extract_abuse_email(&response.entities),
                            name: extract_registrar_name(&response.entities)
                        }),
                        ..node
                    }
                )
            } else {
                Some(Node {
                    domain: Some(domain),
                    ..node
                })
            }
        } else {
            Some(node)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod lookup_host_node_tests {
    use test_support::*;
    use super::*;
    use crate::data::{HostNode, InfrastructureProvider};
    use crate::mountebank::*;

    #[test]
    fn updates_host_node_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let expected = Some(
            output_host_node(
                "host.dodgyaf.com",
                "Reg Six",
                "abuse@regfive.zzz",
                Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap(),
                "10.10.10.10",
                "Acme Hosting",
                "abuse@acmehost.zzz",
            )
        );

        let actual = tokio_test::block_on(
            lookup_host_node(
                Arc::new(bootstrap),
                input_host_node("host.dodgyaf.com", "10.10.10.10"),
            )
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn returns_none_if_no_host_node_provided() {
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let actual = tokio_test::block_on(lookup_host_node(Arc::new(bootstrap), None));

        assert_eq!(None, actual);
    }

    #[test]
    fn returns_updated_host_node_when_node_does_not_have_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let expected = Some(
            output_host_node_sans_domain(
                "host.dodgyaf.com",
                "10.10.10.10",
                "Acme Hosting",
                "abuse@acmehost.zzz",
            )
        );

        let actual = tokio_test::block_on(
            lookup_host_node(
                Arc::new(bootstrap),
                input_host_node_sans_domain("host.dodgyaf.com", "10.10.10.10"),
            )
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn returns_updated_host_node_when_node_does_not_have_ip_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let expected = Some(
            output_host_node_sans_ip_address(
                "host.dodgyaf.com",
                "Reg Six",
                "abuse@regfive.zzz",
                Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap(),
            )
        );

        let actual = tokio_test::block_on(
            lookup_host_node(
                Arc::new(bootstrap),
                input_host_node_sans_ip_address("host.dodgyaf.com"),
            )
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn returns_updated_host_node_when_ip_does_not_have_entities() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = tokio_test::block_on(get_bootstrap());

        let expected = Some(
            output_host_node_sans_infrastructure_provider(
                "host.dodgyaf.com",
                "192.168.10.10",
                "Reg Six",
                "abuse@regfive.zzz",
                Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap(),
            )
        );

        let actual = tokio_test::block_on(
            lookup_host_node(
                Arc::new(bootstrap),
                input_host_node("host.dodgyaf.com", "192.168.10.10"),
            )
        );

        assert_eq!(expected, actual);
    }

    fn input_host_node(host_name: &str, ip_address: &str) -> Option<HostNode> {
        Some(HostNode::new(Some(host_name), Some(ip_address)).unwrap())
    }

    fn input_host_node_sans_domain(host_name: &str, ip_address: &str) -> Option<HostNode> {
        Some(HostNode{
            domain: None,
            host: Some(host_name.into()),
            infrastructure_provider: None,
            ip_address: Some(ip_address.into()),
            registrar: None
        })
    }

    fn input_host_node_sans_ip_address(host_name: &str) -> Option<HostNode> {
        Some(HostNode::new(Some(host_name), None).unwrap())
    }

    fn output_host_node(
        host_name: &str,
        registrar_name: &str,
        registrar_abuse_email_address: &str,
        registration_date: DateTime<Utc>,
        ip_address: &str,
        infrastructure_provider_name: &str,
        infrastructure_provider_email_address: &str,
    ) -> HostNode {
        HostNode {
            domain: domain_object(host_name, Some(registration_date)),
            host: Some(host_name.into()),
            infrastructure_provider: infrastructure_provider_instance(
                infrastructure_provider_name, Some(infrastructure_provider_email_address)
            ),
            ip_address: Some(ip_address.into()),
            registrar: registrar_object(registrar_name, Some(registrar_abuse_email_address)),
        }
    }

    fn output_host_node_sans_domain(
        host_name: &str,
        ip_address: &str,
        infrastructure_provider_name: &str,
        infrastructure_provider_email_address: &str,
    ) -> HostNode {
        HostNode {
            domain: None,
            host: Some(host_name.into()),
            infrastructure_provider: infrastructure_provider_instance(
                infrastructure_provider_name, Some(infrastructure_provider_email_address)
            ),
            ip_address: Some(ip_address.into()),
            registrar: None
        }
    }

    fn output_host_node_sans_ip_address(
        host_name: &str,
        registrar_name: &str,
        registrar_abuse_email_address: &str,
        registration_date: DateTime<Utc>,
    ) -> HostNode {
        HostNode {
            domain: domain_object(host_name, Some(registration_date)),
            host: Some(host_name.into()),
            infrastructure_provider: None,
            ip_address: None,
            registrar: registrar_object(registrar_name, Some(registrar_abuse_email_address)),
        }
    }

    fn output_host_node_sans_infrastructure_provider(
        host_name: &str,
        ip_address: &str,
        registrar_name: &str,
        registrar_abuse_email_address: &str,
        registration_date: DateTime<Utc>,
    ) -> HostNode {
        HostNode {
            domain: domain_object(host_name, Some(registration_date)),
            host: Some(host_name.into()),
            infrastructure_provider: None,
            ip_address: Some(ip_address.into()),
            registrar: registrar_object(registrar_name, Some(registrar_abuse_email_address)),
        }
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "host.dodgyaf.com",
                    None,
                    "Reg Six",
                    "abuse@regfive.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                ),
            ]
        );

        setup_ip_v4_server(vec![
            IpServerConfig::response_200(
                "10.10.10.10",
                None,
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")])
            ),
            IpServerConfig::response_200(
                "192.168.10.10",
                None,
                ("192.168.0.0", "192.168.255.255"),
                None
            ),
        ]);
    }

    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
    ) ->  Option<Domain> {
        Some(
            Domain {
                category: DomainCategory::Other,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }

    fn infrastructure_provider_instance(
        name: &str, abuse_email_address: Option<&str>
    ) -> Option<InfrastructureProvider> {
        Some(
            InfrastructureProvider {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }
}

async fn lookup_host_node(bootstrap: Arc<Bootstrap>, node: Option<HostNode>) -> Option<HostNode> {
    if let Some(host_node) = node {
        let get_domain_data = get_rdap_data(bootstrap.clone(), host_node.domain.as_ref().map(|dom| dom.name.clone()));
        let get_ip_data = get_rdap_ip_data(bootstrap.clone(), host_node.ip_address.as_ref().map(|val| val.clone()));

        let (domain_response, ip_response) = tokio::join!(get_domain_data, get_ip_data);

        let (domain, registrar) = if let Some(domain_data) = domain_response {
            let domain = host_node.domain.map(|dom| {
                Domain {
                    registration_date: extract_registration_date(&domain_data.events),
                    ..dom
                }
            });
            let registrar = Some(Registrar {
                abuse_email_address: extract_abuse_email(&domain_data.entities),
                name: extract_registrar_name(&domain_data.entities)
            });
            (domain, registrar)
        } else {
            (host_node.domain, None)
        };

        let infrastructure_provider = if let Some(ip_data) = ip_response {
            if let Some(ip_data_entities) = ip_data.entities {
                Some(InfrastructureProvider {
                    abuse_email_address: extract_abuse_email(&ip_data_entities),
                    name: extract_provider_name(&ip_data_entities)
                })
            } else {
                None
            }
        } else {
            None
        };

        Some(
            HostNode {
                domain,
                registrar,
                infrastructure_provider,
                ..host_node
            }
        )
    } else {
        None
    }
}

#[cfg(test)]
mod lookup_email_address_tests {
    use super::*;
    use crate::mountebank::*;
    use crate:: data::Registrar;
    use test_support::*;

    #[test]
    fn updates_email_address_data_with_rdap_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(updated_email_address_data(), actual);
    }

    #[test]
    fn does_not_update_if_there_is_no_domain() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data_without_domain();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(email_address_data_without_domain(), actual);
    }

    #[test]
    fn does_not_update_if_no_servers_available() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data_without_rdap_servers();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(email_address_data_without_rdap_servers(), actual);
    }

    #[test]
    fn does_not_update_if_server_returns_404() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_404_impostor();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(email_address_data(), actual);
    }

    #[test]
    fn does_not_update_if_email_address_data_already_has_registrar() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data_with_populated_registrar();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(email_address_data_with_populated_registrar(), actual);
    }

    #[test]
    fn does_not_update_if_email_address_domain_open_email_provider() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let data = email_address_data_with_open_email_provider();

        let actual = tokio_test::block_on(lookup_email_address(Arc::new(bootstrap), data));

        assert_eq!(email_address_data_with_open_email_provider(), actual);
    }

    pub fn setup_404_impostor() {
        setup_dns_server(
            vec![
            DnsServerConfig::response_404("fake.net"),
            ]
        );
    }

    fn email_address_data() -> EmailAddressData {
        EmailAddressData {
            address: "someone@fake.net".into(),
            domain: domain_object("fake.net", None, DomainCategory::Other),
            registrar: None,
        }
    }

    fn updated_email_address_data() -> EmailAddressData {
        EmailAddressData {
            address: "someone@fake.net".into(),
            domain: domain_object(
                "fake.net",
                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                DomainCategory::Other,
            ),
            registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
        }
    }

    fn email_address_data_without_domain() -> EmailAddressData {
        EmailAddressData {
            address: "someone@fake.net".into(),
            domain: None,
            registrar: None,
        }
    }

    fn email_address_data_without_rdap_servers() -> EmailAddressData {
        EmailAddressData {
            address: "someone@fake.unobtainium".into(),
            domain: domain_object("fake.unobtainium", None, DomainCategory::Other),
            registrar: None,
        }
    }

    fn email_address_data_with_populated_registrar() -> EmailAddressData {
        EmailAddressData {
            address: "someone@fake.net".into(),
            domain: domain_object("fake.net", None, DomainCategory::Other),
            registrar: Some(
                Registrar {
                    abuse_email_address: None,
                    name: None,
                }
            )
        }
    }

    fn email_address_data_with_open_email_provider() -> EmailAddressData {
            EmailAddressData {
                address: "someone@fake.net".into(),
                domain: domain_object("fake.net", None, DomainCategory::OpenEmailProvider),
                registrar: None
            }
    }

    fn domain_object(
        name: &str,
        registration_date: Option<DateTime<Utc>>,
        category: DomainCategory
    ) ->  Option<Domain> {
        Some(
            Domain {
                category,
                name: name.into(),
                registration_date,
                abuse_email_address: None
            }
        )
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(
            Registrar {
                name: Some(name.into()),
                abuse_email_address: abuse_email_address.map(String::from)
            }
        )
    }
}

async fn lookup_email_address(
    bootstrap: Arc<Bootstrap>, data: EmailAddressData
) -> EmailAddressData {
    if let EmailAddressData {
        domain: Some(
                    Domain {name, category: DomainCategory::Other, ..}
                ),
        registrar: None,
        ..
    } = &data {
        if let Some(response) = get_rdap_data(bootstrap, Some(name.into())).await {
            let registrar_name = extract_registrar_name(&response.entities);
            let abuse_email_address = extract_abuse_email(&response.entities);
            let registration_date = extract_registration_date(&response.events);

            let domain = Domain { registration_date, ..data.domain.unwrap() };

            let registrar = Registrar { name: registrar_name, abuse_email_address, };

            EmailAddressData { domain: Some(domain), registrar: Some(registrar), ..data }
        } else {
            data
        }
    } else {
        data
    }
}

#[cfg(test)]
mod get_rdap_data_tests {
    use super::*;
    use test_support::*;
    use crate::mountebank::*;

    #[test]
    fn generalises_domain_name_until_match_is_found() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert_handle(Arc::clone(&bootstrap), Some("foo.bar.baz.biz.net"), "DOM-BIZ");
        assert_handle(Arc::clone(&bootstrap), Some("foo.bar.baz.buzz.net"), "DOM-BUZZ");
        assert_handle(Arc::clone(&bootstrap), Some("foo.bar.baz.boz.net"), "DOM-BOZ");
        assert_none(Arc::clone(&bootstrap), Some("un.ob.tai.nium.net"));
    }

    #[test]
    fn returns_none_if_no_server() {
        clear_all_impostors();
        setup_bootstrap_server();
        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert_none(Arc::clone(&bootstrap), Some("no_server.zzz"))
    }

    #[test]
    fn returns_none_if_no_domain_name() {
        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert_none(Arc::clone(&bootstrap), None)
    }

    fn assert_handle(bootstrap: Arc<Bootstrap>, domain_name_option: Option<&str>, expected_handle: &str) {
        let domain = tokio_test::block_on(
            get_rdap_data(Arc::clone(&bootstrap), domain_name_option.map(|val| val.into()))
        ).unwrap();

        assert_eq!(String::from(expected_handle), domain.handle.unwrap())
    }

    fn assert_none(bootstrap: Arc<Bootstrap>, domain_name_option: Option<&str>) {
        let result = tokio_test::block_on(
            get_rdap_data(Arc::clone(&bootstrap), domain_name_option.map(|val| val.into()))
        );

        assert!(result.is_none())
    }

    fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "foo.bar.baz.biz.net",
                    Some("DOM-BIZ"),
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "bar.baz.buzz.net",
                    Some("DOM-BUZZ"),
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "boz.net",
                    Some("DOM-BOZ"),
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                // Spurious, just to validate that we do not submit the TLD :)
                DnsServerConfig::response_200(
                    "net",
                    Some("DOM-NET"),
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
            ]
        );
    }
}

async fn get_rdap_data(bootstrap: Arc<Bootstrap>, domain_name_option: Option<String>) -> Option<parser::Domain> {
    let mut domain_response: Option<parser::Domain> = None;

    if let Some(domain_name) = domain_name_option {
        if let Some(servers) = bootstrap.dns.find(&domain_name) {
            let client = Client::new();
            let domain_name_parts: Vec<&str> = domain_name.split('.').collect();
            let num_parts = domain_name_parts.len();

            for start_pos in 0..(num_parts - 1) {
                if domain_response.is_none() {
                    let partial_name = &domain_name_parts[start_pos..num_parts].join(".");

                    if let Ok(response) = client.query_domain(&servers[0], partial_name).await {
                        domain_response = Some(response);
                    }
                }
            };
        }
    }

    domain_response
}

#[cfg(test)]
mod get_rdap_ip_data_tests {
    use test_support::*;
    use super::*;
    use crate::mountebank::*;

    #[test]
    fn returns_ip_data_for_the_provided_v4_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        let data = tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("10.10.10.10".into()))).unwrap();

        assert_eq!(String::from("NET-10.0.0.0"), data.handle);
    }

    #[test]
    fn returns_ip_data_for_the_provided_v6_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        let data = tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("abcd::1".into()))).unwrap();

        assert_eq!(String::from("NET-abcd"), data.handle);
    }

    #[test]
    fn returns_none_if_no_response_for_ip_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert!(tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("10.10.10.11".into()))).is_none());
    }

    #[test]
    fn returns_none_if_not_ip_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert!(tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("xxxx".into()))).is_none());
    }

    #[test]
    fn returns_none_if_no_rdap_server_for_ip_address() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert!(tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("20.20.20.20".into()))).is_none());
    }

    #[test]
    fn returns_none_if_rdap_server_entry_has_no_servers() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();

        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert!(tokio_test::block_on(get_rdap_ip_data(bootstrap, Some("30.30.30.30".into()))).is_none());
    }

    #[test]
    fn returns_none_if_ip_address_is_none() {
        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert!(tokio_test::block_on(get_rdap_ip_data(bootstrap, None)).is_none());
    }

    fn setup_impostors() {
        setup_ip_v4_server(vec![
            IpServerConfig::response_200(
                "10.10.10.10",
                Some("NET-10.0.0.0"),
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "abuse", "abuse@acmehost.zzz")])
            ),
            IpServerConfig::response_404("10.10.10.11"),
            IpServerConfig::response_200(
                "192.168.10.10",
                Some("NET-192.168.10.10"),
                ("192.168.0.0", "192.168.255.255"),
                Some(&[("Hackme Hosting", "abuse", "abuse@hackmehost.zzz")])
            ),
        ]);

        setup_ip_v6_server(vec![
            IpServerConfig::response_200(
                "abcd::1",
                Some("NET-abcd"),
                ("abcd::1", "abcd::ffff"),
                Some(&[("Acme V6 Hosting", "abuse", "abuse@acmev6host.zzz")])
            ),
            IpServerConfig::response_200(
                "bcde::1",
                Some("NET-bcde"),
                ("bcde::1", "bcde::ffff"),
                Some(&[("Hackme V6 Hosting", "abuse", "abuse@hackmev6host.zzz")])
            ),
        ])
    }
}

async fn get_rdap_ip_data(
    bootstrap: Arc<Bootstrap>,
    ip_as_string_option: Option<String>
) -> Option<parser::IpNetwork> {
    if ip_as_string_option.is_none() {
        return None
    }

    let ip_as_string = ip_as_string_option.unwrap();
    let parsed_ip_option = match std::net::Ipv4Addr::from_str(&ip_as_string) {
        Ok(v4_ip) => Some(std::net::IpAddr::from(v4_ip)),
        Err(_) => {
            match std::net::Ipv6Addr::from_str(&ip_as_string) {
                Ok(v6_ip) => Some(std::net::IpAddr::from(v6_ip)),
                Err(_) => None
            }
        }
    };

    if let Some(ip) = parsed_ip_option {
        if let Some(servers) = bootstrap.ip.find(ip) {
            if !servers.is_empty() {
                let client = Client::new();
                client.query_ip(&servers[0], ip).await.ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}


#[cfg(test)]
mod extract_registrar_name_tests {
    use super::*;

    #[test]
    fn extracts_registrar_name() {
        let entities = entities_including_registrar();

        assert_eq!(
            Some(String::from("Reg One")),
            extract_registrar_name(&entities)
        );
    }

    #[test]
    fn no_registrar_role() {
        let entities = entities_no_registrar();

        assert_eq!(
            None,
            extract_registrar_name(&entities)
        );
    }

    #[test]
    fn registrar_role_no_full_name() {
        let entities = entities_including_registrar_no_fn();

        assert_eq!(
            None,
            extract_registrar_name(&entities)
        );
    }

    fn entities_including_registrar() -> Vec<parser::Entity> {
        vec![
            build_entity(
                Some(vec![parser::Role::Registrant, parser::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    parser::Role::Noc,
                    parser::Role::Registrar,
                    parser::Role::Sponsor
                ]),
                ("fn", "Reg One"),
            ),
            build_entity(
                Some(vec![parser::Role::Noc, parser::Role::Sponsor]),
                ("fn", "Not Reg Two")
            )
        ]
    }

    fn entities_no_registrar() -> Vec<parser::Entity> {
        vec![
            build_entity(
                Some(vec![parser::Role::Registrant, parser::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    parser::Role::Noc,
                    parser::Role::Sponsor
                ]),
                ("fn", "Not Reg Two"),
            ),
            build_entity(
                Some(vec![
                    parser::Role::Noc,
                    parser::Role::Sponsor,
                ]),
                ("fn", "Not Reg Three")
            )
        ]
    }

    fn entities_including_registrar_no_fn() -> Vec<parser::Entity> {
        vec![
            build_entity(
                Some(vec![parser::Role::Registrant, parser::Role::Sponsor]),
                ("fn", "Not Reg One"),
            ),
            build_entity(
                Some(vec![
                    parser::Role::Noc,
                    parser::Role::Registrar,
                    parser::Role::Sponsor
                ]),
                ("not-fn", "Reg One"),
            ),
            build_entity(
                Some(vec![parser::Role::Noc, parser::Role::Sponsor]),
                ("fn", "Not Reg Two")
            )
        ]
    }

    fn build_entity(
        roles: Option<Vec<parser::Role>>,
        additional_vcard_item: (&str, &str)
    ) -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![
                build_jcard_item("foo", "bar"),
                build_jcard_item(additional_vcard_item.0, additional_vcard_item.1),
                build_jcard_item("baz", "biz"),
            ]
        );

        let handle: Option<String> = None;

        parser::Entity {
            roles,
            vcard_array: Some(vcard_array),
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, value: &str) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: vec![json!(value)]

        }
    }
}

fn extract_registrar_name(entities: &[parser::Entity]) -> Option<String> {
    if let Some(entity) = find_registrar_entity(entities) {
        extract_full_name(entity)
    } else {
        None
    }
}

#[cfg(test)]
mod extract_provider_name_tests {
    use super::*;

    #[test]
    fn returns_name_of_most_recent_registrant() {
        let entity = registrant_entity();

        assert_eq!(
            Some(String::from("Provider XXX")),
            extract_provider_name(&[entity])
        );
    }

    fn registrant_entity() -> parser::Entity {
        let last_changed_date = chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 1, 23, 14, 10, 20)
            .unwrap();

        build_entity(
            &[parser::Role::Registrant],
            None,
            Some(&[("fn", &["Provider XXX"])]),
            Some(last_changed_date),
        )
    }

    fn build_entity(
        roles: &[parser::Role],
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        parser::Entity {
            roles: Some(roles.to_vec()),
            vcard_array,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events: Some(build_events(last_changed.unwrap())),
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn extract_provider_name(entities: &[parser::Entity]) -> Option<String> {
    let registrant_entities = extract_eligible_registrant_entities(entities);
    find_most_recent_full_name(registrant_entities)
}

#[cfg(test)]
mod extract_eligible_registrant_entities_tests {
    use super::*;

    #[test]
    fn returns_an_empty_array_if_entities_are_empty() {
        assert!(extract_eligible_registrant_entities(&[]).is_empty());
    }

    #[test]
    fn returns_registrant_entities_with_names() {
        let entities = &[
            registrant_entity("H-R-1"),
            non_registrant_entity("H-NR-2"),
            registrant_entity("H-R-2"),
            registrant_entity_no_full_name("H-R-NFN")
        ];

        let expected_handles: Vec<String> = vec!["H-R-1".into(), "H-R-2".into()];

        let actual_handles = extract_handles(extract_eligible_registrant_entities(entities));

        assert_eq!(expected_handles, actual_handles);
    }

    fn registrant_entity(handle: &str) -> parser::Entity {
        build_entity(
            handle,
            Some(
                &[
                    parser::Role::Sponsor,
                    parser::Role::Registrant,
                    parser::Role::Reseller,
                ]
            ),
            None,
            Some(&[("fn", &["does not matter"])]),
            None
        )
    }

    fn registrant_entity_no_full_name(handle: &str) -> parser::Entity {
        build_entity(
            handle,
            Some(
                &[
                    parser::Role::Sponsor,
                    parser::Role::Registrant,
                    parser::Role::Reseller,
                ]
            ),
            None,
            None,
            None
        )
    }

    fn non_registrant_entity(handle: &str) -> parser::Entity {
        build_entity(
            handle,
            Some(
                &[
                    parser::Role::Sponsor,
                    parser::Role::Reseller,
                ]
            ),
            None,
            Some(&[("fn", &["does not matter"])]),
            None
        )
    }

    fn build_entity(
        handle: &str,
        roles: Option<&[parser::Role]>,
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        let events = if let Some(date) = last_changed {
            Some(build_events(date))
        } else {
            None
        };

        parser::Entity {
            roles: roles.map(|r| r.to_vec()),
            vcard_array,
            handle: Some(handle.into()),
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }

    fn extract_handles(entities: Vec<&parser::Entity>) -> Vec<String> {
        entities
            .into_iter()
            .flat_map(|entity| entity.handle.clone())
            .collect()
    }
}

fn extract_eligible_registrant_entities(entities: &[parser::Entity]) -> Vec<&parser::Entity> {
    entities
        .into_iter()
        .filter(|entity| {
            is_registrant(entity) && has_full_name(entity)
        })
        .collect()
}

#[cfg(test)]
mod is_registrant_tests {
    use super::*;

    #[test]
    fn is_false_if_entity_has_no_roles() {
        let entity = no_roles_entity();

        assert!(!is_registrant(&entity))
    }

    #[test]
    fn is_false_if_entity_has_empty_roles() {
        let entity = empty_roles_entity();

        assert!(!is_registrant(&entity))
    }

    #[test]
    fn is_false_if_entity_does_not_have_registrant_role() {
        let entity = non_registrant_entity();

        assert!(!is_registrant(&entity))
    }

    #[test]
    fn is_true_if_entity_has_registrant_role() {
        let entity = registrant_entity();

        assert!(is_registrant(&entity))
    }

    fn no_roles_entity() -> parser::Entity {
        build_entity("does not matter", None, None, None, None)
    }

    fn empty_roles_entity() -> parser::Entity {
        build_entity("does not matter", Some(&[]), None, None, None)
    }

    fn non_registrant_entity() -> parser::Entity {
        build_entity(
            "does not matter",
            Some(&[
                parser::Role::Sponsor,
                parser::Role::Reseller,
            ]),
            None,
            None,
            None
        )
    }

    fn registrant_entity() -> parser::Entity {
        build_entity(
            "does not matter",
            Some(&[
                parser::Role::Sponsor,
                parser::Role::Registrant,
                parser::Role::Reseller,
            ]),
            None,
            None,
            None
        )
    }

    fn build_entity(
        handle: &str,
        roles: Option<&[parser::Role]>,
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        let events = if let Some(date) = last_changed {
            Some(build_events(date))
        } else {
            None
        };

        parser::Entity {
            roles: roles.map(|r| r.to_vec()),
            vcard_array,
            handle: Some(handle.into()),
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }

}

fn is_registrant(entity: &parser::Entity) -> bool {
    if let Some(roles) = &entity.roles {
        roles.contains(&parser::Role::Registrant)
    } else {
        false
    }
}

#[cfg(test)]
mod has_full_name_tests {
    use super::*;

    #[test]
    fn returns_false_if_no_cards_array() {
        let entity = entity_with_no_full_name_card();

        assert!(!has_full_name(&entity));
    }

    #[test]
    fn returns_false_if_no_full_name_cards() {
        let entity = entity_without_full_name_cards();

        assert!(!has_full_name(&entity))
    }

    #[test]
    fn returns_false_if_full_name_card_without_values() {
        let entity = entity_with_card_but_no_values();

        assert!(!has_full_name(&entity))
    }

    #[test]
    fn returns_false_if_full_name_card_has_empty_strings_for_values() {
        let entity = entity_with_card_but_empty_string_values();

        assert!(!has_full_name(&entity))
    }

    #[test]
    fn returns_true_if_single_full_name_card() {
        let entity = entity_with_single_full_name_card();

        assert!(has_full_name(&entity))
    }

    #[test]
    fn returns_true_if_multiple_full_name_cards() {
        let entity = entity_with_multiple_full_name_cards();

        assert!(has_full_name(&entity))
    }

    fn entity_with_no_full_name_card() -> parser::Entity {
        build_entity("does not matter", None, None, None, None)
    }

    fn entity_without_full_name_cards() -> parser::Entity {
        build_entity("does not matter", None, None, Some(&[]), None)
    }

    fn entity_with_card_but_no_values() -> parser::Entity {
        build_entity("does not matter", None, None, Some(&[("fn", &[])]), None)
    }

    fn entity_with_card_but_empty_string_values() -> parser::Entity {
        build_entity("does not matter", None, None, Some(&[("fn", &["", ""])]), None)
    }

    fn entity_with_single_full_name_card() -> parser::Entity {
        build_entity("does not matter", None, None, Some(&[("fn", &["My Name"])]), None)
    }

    fn entity_with_multiple_full_name_cards() -> parser::Entity {
        build_entity(
            "does not matter",
            None,
            None,
            Some(&[("fn", &["My Name"]), ("fn", &["My Other Name"])]),
            None
        )
    }

    fn build_entity(
        handle: &str,
        roles: Option<&[parser::Role]>,
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        let events = if let Some(date) = last_changed {
            Some(build_events(date))
        } else {
            None
        };

        parser::Entity {
            roles: roles.map(|r| r.to_vec()),
            vcard_array,
            handle: Some(handle.into()),
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn has_full_name(entity: &parser::Entity) -> bool {
    if let Some(vcard_array) = &entity.vcard_array {
        vcard_array
            .items_by_name("fn")
            .iter()
            .flat_map(|card| {
                card.values
                    .iter()
                    .flat_map(|val| val.as_str())
                    .collect::<Vec<&str>>()
            })
            .any(|value| !value.is_empty())
    } else {
        false
    }
}

#[cfg(test)]
mod find_most_recent_full_name_tests {
    use super::*;

    #[test]
    fn returns_none_if_empty_collection() {
        assert!(find_most_recent_full_name(vec![]).is_none())
    }

    #[test]
    fn returns_the_full_name_of_most_recent_entity() {
        let entity_1 = build_entity_with_name_and_time("E-1", 9);
        let entity_2 = build_entity_with_name_and_time("E-2", 11);
        let entity_3 = build_entity_with_name_and_time("E-3", 10);

        assert_eq!(
            Some(String::from("E-2")),
            find_most_recent_full_name(vec![&entity_1, &entity_2, &entity_3])
        )
    }

    fn build_entity_with_name_and_time(name: &str, seconds: u32) -> parser::Entity {
        let last_changed = chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 1, 23, 14, 10, seconds)
            .unwrap();

        let fullname_values = vec![name];
        let vcard_items = &[("fn", &fullname_values[..])];

        build_entity("does not matter", None, None, Some(vcard_items), Some(last_changed))
    }

    fn build_entity(
        handle: &str,
        roles: Option<&[parser::Role]>,
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        let events = if let Some(date) = last_changed {
            Some(build_events(date))
        } else {
            None
        };

        parser::Entity {
            roles: roles.map(|r| r.to_vec()),
            vcard_array,
            handle: Some(handle.into()),
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn find_most_recent_full_name(mut entities: Vec<&parser::Entity>) -> Option<String> {
    entities.sort_by(|a, b| last_changed_date(a).cmp(&last_changed_date(b)));

    if let Some(last_entity) = entities.last() {
        extract_full_name(last_entity)
    } else {
        None
    }
}

#[cfg(test)]
mod extract_full_name_tests {
    use super::*;

    #[test]
    fn extract_full_name_from_entity() {
        assert_eq!(
            Some(String::from("Reg One")),
            extract_full_name(&entity_with_full_name())
        );
    }

    #[test]
    fn extract_full_name_no_vcard_array() {
        assert_eq!(
            None,
            extract_full_name(&entity_without_vcards())
        );
    }

    #[test]
    fn extract_full_name_no_fn_vcard() {
        assert_eq!(
            None,
            extract_full_name(&entity_without_fn_vcard())
        );
    }

    #[test]
    fn extract_full_name_multiple_fn_vcards() {
        assert_eq!(
            Some("Reg One".into()),
            extract_full_name(&entity_with_multiple_fn_vcards())
        );
    }

    #[test]
    fn extract_full_name_multiple_fn_values() {
        assert_eq!(
            Some("Reg One".into()),
            extract_full_name(
                &build_entity(None, vec![("fn", &["Reg One", "Reg Two"])])
            )
        );
    }

    fn entity_with_full_name() -> parser::Entity {
        build_entity(None, vec![("fn", &["Reg One"])])
    }

    fn build_entity(
        roles: Option<Vec<parser::Role>>,
        additional_items: Vec<(&str, &[&str])>
    ) -> parser::Entity {
        let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
        let mut additional_vcard_items = additional_items.iter().map(|(c_type, c_values)| {
            build_jcard_item(c_type, c_values)
        }).collect();
        let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

        vcard_items.append(&mut additional_vcard_items);
        vcard_items.append(&mut trailing_vcard_items);

        let vcard_array = parser::JCard(parser::JCardType::Vcard, vcard_items);

        let handle: Option<String> = None;

        parser::Entity {
            roles,
            vcard_array: Some(vcard_array),
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }

    fn entity_without_vcards() -> parser::Entity {
        let handle: Option<String> = None;

        parser::Entity {
            roles: None,
            vcard_array: None,
            handle,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn entity_without_fn_vcard() -> parser::Entity {
        build_entity(None, vec![("not-fn", &["Reg One"])])
    }

    fn entity_with_multiple_fn_vcards() -> parser::Entity {
        build_entity(None, vec![("fn", &["Reg One"]), ("fn", &["Reg Two"])])
    }
}

fn extract_full_name(entity: &parser::Entity) -> Option<String> {
    if let Some(vcards) = &entity.vcard_array {
        let full_name_items = vcards.items_by_name("fn");

        full_name_items
            .first()
            .map(|item| {
                item.values.first().unwrap().as_str().unwrap().into()
            })
    } else {
        None
    }
}

#[cfg(test)]
mod extract_abuse_email_tests {
    use super::*;

    #[test]
    fn returns_email_address_of_most_recently_updated_abuse_entity() {
        assert_eq!(
            Some(String::from("abuse@test.zzz")),
            extract_abuse_email(&[entity_with_abuse_entity()])
        );
    }

    fn entity_with_abuse_entity() -> parser::Entity {
        let last_changed_date = chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 1, 23, 14, 10, 20)
            .unwrap();

        build_entity(
            &[parser::Role::Registrant],
            Some(vec![abuse_entity()]),
            None,
            Some(last_changed_date),
        )
    }

    fn abuse_entity() -> parser::Entity {
        let last_changed_date = chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 2, 23, 14, 10, 20)
            .unwrap();

        build_entity(
            &[parser::Role::Abuse],
            Some(vec![]),
            Some(&[("email", &["abuse@test.zzz"])]),
            Some(last_changed_date),
        )
    }

    fn build_entity(
        roles: &[parser::Role],
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>,
        last_changed: Option<DateTime<FixedOffset>>
    ) -> parser::Entity {
        let vcard_array = if let Some(additional) = additional_items {
            let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];
            let mut additional_vcard_items = additional.iter().map(|(c_type, c_values)| {
                build_jcard_item(c_type, c_values)
            }).collect();
            let mut trailing_vcard_items = vec![build_jcard_item("baz", &["biz"])];

            vcard_items.append(&mut additional_vcard_items);
            vcard_items.append(&mut trailing_vcard_items);

            Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
        } else {
            None
        };

        parser::Entity {
            roles: Some(roles.to_vec()),
            vcard_array,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events: Some(build_events(last_changed.unwrap())),
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn extract_abuse_email(entities: &[parser::Entity]) -> Option<String> {
    let abuse_entities = extract_eligible_abuse_entities(entities);
    find_most_recent_email_address(&abuse_entities)
}

#[cfg(test)]
mod extract_registration_date_tests {
    use super::*;

    #[test]
    fn returns_registration_date() {
        let events = parser::Events(vec![
            non_registration_event(), registration_event(), non_registration_event()
        ]);
        let expected_registration_date = chrono::Utc
            .with_ymd_and_hms(2022, 12, 11, 12, 5, 30)
            .single()
            .unwrap();

        assert_eq!(
            Some(expected_registration_date),
            extract_registration_date(&events)
        );
    }

    #[test]
    fn returns_none_if_no_registration_event() {
        let events = parser::Events(vec![
            non_registration_event(), non_registration_event()
        ]);

        assert!(extract_registration_date(&events).is_none());
    }

    fn registration_event() -> parser::Event {
        let event_date = time_zone()
            .with_ymd_and_hms(2022, 12, 11, 14, 5, 30)
            .single()
            .unwrap();

        parser::Event {
            event_actor: None,
            event_action: parser::EventAction::Registration,
            event_date,
            links: None,
        }
    }

    fn non_registration_event() -> parser::Event {

        let event_date = time_zone()
            .with_ymd_and_hms(2022, 12, 25, 16, 5, 30)
            .single()
            .unwrap();

        parser::Event {
            event_actor: None,
            event_action: parser::EventAction::Locked,
            event_date,
            links: None,
        }
    }

    fn time_zone() -> chrono::FixedOffset {
        chrono::FixedOffset::east_opt(2 * 3600).unwrap()
    }
}

fn extract_registration_date(events: &parser::Events) -> Option<DateTime<Utc>> {
    events
        .action_date(parser::EventAction::Registration)
        .map(|date| date.into())
}

#[cfg(test)]
mod find_registrar_entity_tests {
    use super::*;

    #[test]
    fn extracts_registrar_name() {
        let entities = vec![
            non_registrar_entity(),
            registrar_entity_1(),
            non_registrar_entity(),
        ];

        compare(&registrar_entity_1(), find_registrar_entity(&entities).unwrap());
    }

    #[test]
    fn multiple_registrar_entries_returns_last_registrar() {
        let entities = vec![
            non_registrar_entity(),
            registrar_entity_1(),
            registrar_entity_2(),
            non_registrar_entity(),
        ];

        compare(&registrar_entity_2(), find_registrar_entity(&entities).unwrap());
    }

    #[test]
    fn no_registrar_role() {
        let entities = vec![
            non_registrar_entity(),
            non_registrar_entity(),
        ];

        assert!(find_registrar_entity(&entities).is_none());
    }

    fn non_registrar_entity() -> parser::Entity {
        build_entity(&[
            parser::Role::Noc,
            parser::Role::Sponsor
        ])
    }

    fn registrar_entity_1() -> parser::Entity {
        build_entity(&[
            parser::Role::Noc,
            parser::Role::Registrar,
            parser::Role::Sponsor
        ])
    }

    fn registrar_entity_2() -> parser::Entity {
        build_entity(&[
            parser::Role::Noc,
            parser::Role::Registrar,
        ])
    }

    fn build_entity(
        roles: &[parser::Role],
    ) -> parser::Entity {
        parser::Entity {
            roles: Some(roles.to_vec()),
            vcard_array: None,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn compare(expected: &parser::Entity, actual: &parser::Entity) {
        // Use the assigned roles as a unique 'id'
        assert_eq!(expected.roles, actual.roles);
    }
}

fn find_registrar_entity(entities: &[parser::Entity]) -> Option<&parser::Entity> {
    let mut registrar_entities: Vec<&parser::Entity> = entities
        .iter()
        .filter(|e| {
            if let Some(roles) = &e.roles {
                roles.contains(&parser::Role::Registrar)
            } else {
                false
            }
        })
    .collect();

    registrar_entities.pop()
}

#[cfg(test)]
mod extract_eligible_abuse_entities_tests {
    use parser::{JCard, JCardItem, JCardItemDataType, JCardType, Role};
    use parser::Role::{Abuse, Administrative, Registrant};
    use super::*;

    #[test]
    fn returns_flattened_collection_of_all_abuse_entities_with_email_addresses() {
        let entities = &[
            build_entity_set(
                ("h-1", vec![Administrative, Registrant]),
                vec![
                    ("h-2", vec![Administrative, Abuse, Registrant], true),
                    ("h-3", vec![Administrative, Registrant], true),
                    ("h-4", vec![Abuse], true),
                ],
                true
            ),
            build_entity_set(
                ("h-5", vec![Administrative, Abuse, Registrant]),
                vec![
                    ("h-6", vec![Administrative, Registrant], true),
                    ("h-7", vec![Administrative, Abuse, Registrant], false),
                    ("h-8", vec![Administrative, Abuse, Registrant], true),
                ],
                true
            ),
            build_entity_set(
                ("h-9", vec![Administrative, Abuse, Registrant]),
                vec![
                    ("h-10", vec![Administrative, Abuse, Registrant], true),
                ],
                false
            ),
        ];

        let expected_entity_handles: Vec<String> = vec![
            "h-2".into(), "h-4".into(), "h-5".into(), "h-8".into(), "h-10".into()
        ];

        let actual_entity_handles: Vec<String> = extract_eligible_abuse_entities(entities)
            .into_iter()
            .map(|entity| entity.handle.clone().unwrap())
            .collect();

        assert_eq!(expected_entity_handles, actual_entity_handles);
    }

    fn build_entity_set(
        top_level: (&str, Vec<Role>),
        lower_level: Vec<(&str, Vec<Role>, bool)>,
        has_email: bool
    ) -> parser::Entity {
        let lower_level_entities = lower_level
            .into_iter()
            .map(|(handle, roles, has_email)| build_entity(handle, roles, vec![], has_email))
            .collect();

        let (handle, roles) = top_level;

        build_entity(handle, roles, lower_level_entities, has_email)
    }

    fn build_entity(
        handle: &str,
        roles: Vec<parser::Role>,
        entities: Vec<parser::Entity>,
        has_email: bool
        ) -> parser::Entity {
        let mut vcard_items = vec![build_jcard_item("foo", &["bar"])];

        if has_email {
            vcard_items.append(
                &mut vec![build_jcard_item("email", &[&format!("{handle}@test.zzz")])]
            )
        }

        vcard_items.append(
            &mut vec![build_jcard_item("baz", &["biz"])]
        );

        parser::Entity {
            roles: Some(roles),
            vcard_array: Some(JCard(JCardType::Vcard, vcard_items)),
            handle: Some(String::from(handle)),
            as_event_actor: None,
            public_ids: None,
            entities: Some(entities),
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn extract_eligible_abuse_entities(entities: &[parser::Entity]) -> Vec<&parser::Entity> {
    let all_entities: Vec<&parser::Entity> = entities.
        iter()
        .flat_map(|entity| {
            all_entities_from(entity)
        })
        .collect();

    all_entities
        .into_iter()
        .filter(|entity| is_abuse_entity(entity) && has_email_address(entity))
        .collect()
}

#[cfg(test)]
mod all_entities_from_tests {
    use super::*;

    #[test]
    fn entity_has_children_returns_all_entities() {
        let child_entity_1_child = build_entity("c-1-c", None);
        let child_entity_1 = build_entity("c-1", Some(vec![child_entity_1_child]));

        let child_entity_2_child = build_entity("c-2-c", None);
        let child_entity_2 = build_entity("c-2", Some(vec![child_entity_2_child]));

        let entity = build_entity("e-1", Some(vec![child_entity_1, child_entity_2]));

        let expected_handles = vec![
            String::from("e-1"),
            String::from("c-1"),
            String::from("c-2"),
        ];

        assert_eq!(expected_handles, handles(all_entities_from(&entity)));
    }

    #[test]
    fn entity_has_no_children_just_returns_entity() {
        let entity = build_entity("e-1", Some(vec![]));

        let expected_handles = vec![String::from("e-1")];

        let actual_handles = handles(all_entities_from(&entity));

        assert_eq!(expected_handles, actual_handles);
    }

    #[test]
    fn entity_has_no_children_vec_just_returns_entity() {
        let entity = build_entity("e-1", None);

        let expected_handles = vec![String::from("e-1")];

        let actual_handles = handles(all_entities_from(&entity));

        assert_eq!(expected_handles, actual_handles);
    }

    fn build_entity(
        handle: &str,
        entities: Option<Vec<parser::Entity>>,
    ) -> parser::Entity {

        parser::Entity {
            roles: None,
            vcard_array: None,
            handle: Some(String::from(handle)),
            as_event_actor: None,
            public_ids: None,
            entities,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn handles(entities: Vec<&parser::Entity>) -> Vec<String> {
        entities
            .iter()
            .map(|entity| entity.handle.clone().unwrap())
            .collect()
    }

}

fn all_entities_from(entity: &parser::Entity) -> Vec<&parser::Entity> {
    let mut output = vec![entity];
    if let Some(entities) = &entity.entities {
        let mut child_entities = entities
            .iter()
            .map(|entity| entity)
            .collect();

        output.append(&mut child_entities);
    } 

    output
}

#[cfg(test)]
mod find_most_recent_email_address_tests {
    use super::*;

    #[test]
    fn returns_none_if_no_entities() {
        let entities: Vec<&parser::Entity> = vec![];

        assert!(find_most_recent_email_address(&entities).is_none())
    }

    #[test]
    fn returns_the_email_address_of_most_recently_updated() {
        let entities = &[
            &build_entity(10, &["abuse@regone.zzz"]),
            &build_entity(30, &["abuse@regthree.zzz", "otherabuse@regthree.zzz"]),
            &build_entity(20, &["abuse@regtwo.zzz"]),
        ];

        assert_eq!(
            Some(String::from("abuse@regthree.zzz")),
            find_most_recent_email_address(entities)
        );
    }

    #[test]
    fn returns_email_address_even_without_last_changed_date() {
        let entities = &[
            &build_entity(10, &["abuse@regone.zzz"]),
            &build_entity_without_last_changed_event(&["abuse@regthree.zzz"]),
            &build_entity(20, &["abuse@regtwo.zzz"]),
        ];

        assert_eq!(
            Some(String::from("abuse@regtwo.zzz")),
            find_most_recent_email_address(entities)
        );
    }

    fn build_entity(seconds: u32, email_addresses: &[&str]) -> parser::Entity {
        let last_changed = chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 1, 23, 14, 10, seconds)
            .unwrap();

        parser::Entity {
            roles: None,
            vcard_array: build_vcard_array(email_addresses),
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: Some(build_events(last_changed)),
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_entity_without_last_changed_event(email_addresses: &[&str]) -> parser::Entity{
        parser::Entity {
            roles: None,
            vcard_array: build_vcard_array(email_addresses),
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events(event_date: DateTime<FixedOffset>) -> parser::Events {
        let event = parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date,
            links: None
        };

        parser::Events(vec![event])
    }

    fn build_vcard_array(email_addresses: &[&str]) -> Option<parser::JCard> {
        let vcard_items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", email_addresses),
            build_jcard_item("baz", &["biz"]),
        ];

        Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn find_most_recent_email_address(entities: &[&parser::Entity]) -> Option<String> {
    // TODO in find_most_recent_full_name we pass in Vec rather than a slice - may be 
    // nice to have the same signature n both places
    let mut working_entities = entities.to_vec();

    working_entities.sort_by(|a, b| last_changed_date(a).cmp(&last_changed_date(b)));

    if let Some(last_entity) = working_entities.last() {
        get_email_address(last_entity)
    } else {
        None
    }
}

#[cfg(test)]
mod last_changed_date_tests {
    use super::*;

    #[test]
    fn returns_the_last_changed_date() {
        let entity = build_entity_with_events();

        assert_eq!(Some(build_date(10)), last_changed_date(&entity));
    }

    #[test]
    fn returns_none_if_no_events() {
        let entity = build_entity_without_events();

        assert_eq!(None, last_changed_date(&entity));
    }

    fn build_entity_with_events() -> parser::Entity {
        parser::Entity {
            roles: None,
            vcard_array: None,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: Some(build_events()),
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_entity_without_events() -> parser::Entity {
        parser::Entity {
            roles: None,
            vcard_array: None,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_events() -> parser::Events {
        parser::Events(vec![
            build_last_changed_event(),
        ])
    }

    fn build_last_changed_event() -> parser::Event {
        parser::Event {
            event_actor: None,
            event_action: parser::EventAction::LastChanged,
            event_date: build_date(10),
            links: None
        }
    }

    fn build_date(seconds: u32) -> DateTime<FixedOffset> {
        chrono::FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2023, 1, 23, 14, 10, seconds)
            .unwrap()
    }
}

fn last_changed_date(entity: &parser::Entity) -> Option<DateTime<FixedOffset>> {
    if let Some(events) = &entity.events {
        events.action_date(parser::EventAction::LastChanged)
    } else {
        None
    }
}

#[cfg(test)]
mod get_email_address_tests {
    use super::*;

    #[test]
    fn returns_none_if_no_vcard_array() {
        let entity = build_entity_with_no_vcard_array();

        assert_eq!(None, get_email_address(&entity));
    }

    #[test]
    fn returns_none_if_empty_vcard_array() {
        let entity = build_entity_with_empty_vcard_array();

        assert_eq!(None, get_email_address(&entity));
    }

    #[test]
    fn returns_none_if_no_email_cards() {
        let entity = build_entity_with_no_email_cards();

        assert_eq!(None, get_email_address(&entity));
    }

    #[test]
    fn returns_none_if_email_card_no_values() {
        let entity = build_entity_with_single_email_card_no_values();

        assert_eq!(None, get_email_address(&entity));
    }

    #[test]
    fn returns_none_if_email_card_empty_string() {
        let entity = build_entity_with_single_email_card_empty_string_value();

        assert_eq!(None, get_email_address(&entity));
    }

    #[test]
    fn returns_email_address_if_single_email_card() {
        let entity = build_entity_with_single_email_card();

        assert_eq!(Some(String::from("abuse@regtest.zzz")), get_email_address(&entity));
    }

    #[test]
    fn returns_first_email_address_if_multiple_values() {
        let entity = build_entity_with_single_email_card_multiple_values();

        assert_eq!(Some(String::from("abuse@regtest.zzz")), get_email_address(&entity));
    }

    #[test]
    fn returns_first_email_address_if_multiple_email_cards() {
        let entity = build_entity_with_multiple_email_cards();

        assert_eq!(Some(String::from("abuse@regtest.zzz")), get_email_address(&entity));
    }

    fn build_entity_with_no_vcard_array() -> parser::Entity {
        build_entity(None)
    }

    fn build_entity_with_empty_vcard_array() -> parser::Entity {
        build_entity(Some(parser::JCard(parser::JCardType::Vcard, vec![])))
    }

    fn build_entity_with_no_email_cards() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity_with_single_email_card() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", &["abuse@regtest.zzz"]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity_with_single_email_card_no_values() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", &[]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity_with_single_email_card_empty_string_value() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", &[""]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity_with_single_email_card_multiple_values() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", &["abuse@regtest.zzz", "otherabuse@regtest.zzz"]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity_with_multiple_email_cards() -> parser::Entity {
        let items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", &["abuse@regtest.zzz"]),
            build_jcard_item("email", &["otherabuse@regtest.zzz"]),
            build_jcard_item("baz", &["biz"]),
        ];

        build_entity(Some(parser::JCard(parser::JCardType::Vcard, items)))
    }

    fn build_entity(vcard_array: Option<parser::JCard>) -> parser::Entity {
        parser::Entity {
            roles: None,
            vcard_array,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_vcard_array(email_addresses: &[&str]) -> Option<parser::JCard> {
        let vcard_items = vec![
            build_jcard_item("foo", &["bar"]),
            build_jcard_item("email", email_addresses),
            build_jcard_item("baz", &["biz"]),
        ];

        Some(parser::JCard(parser::JCardType::Vcard, vcard_items))
    }

    fn build_jcard_item(property_name: &str, values: &[&str]) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: values.iter().map(|v| json!(v)).collect()
        }
    }
}

fn get_email_address(entity: &parser::Entity) -> Option<String> {
    // TODO Can we use this in the has_email_address function?
    if let Some(vcard_array) = &entity.vcard_array {
        vcard_array
            .items_by_name("email")
            .into_iter()
            .flat_map(|item| {
                item
                    .values
                    .iter()
                    .flat_map(|v| v.as_str())
                    .collect::<Vec<&str>>()
            })
            .filter(|value| !value.is_empty())
            .collect::<Vec<&str>>()
            .first()
            .map(|address| String::from(*address))
    } else {
        None
    }
}

#[cfg(test)]
mod is_abuse_entity_tests {
    use super::*;
    use test_friendly_rdap_client::parser::Role;
    use test_friendly_rdap_client::parser::Role::{Abuse, Administrative, Technical};

    #[test]
    fn returns_false_if_entity_has_no_roles() {
        let entity = build_entity(None);

        assert!(!is_abuse_entity(&entity));
    }

    #[test]
    fn returns_false_if_entity_has_empty_roles() {
        let entity = build_entity(Some(vec![]));

        assert!(!is_abuse_entity(&entity));
    }

    #[test]
    fn returns_false_if_entity_has_non_abuse_roles() {
        let entity = build_entity(Some(vec![Technical, Administrative]));

        assert!(!is_abuse_entity(&entity));
    }

    #[test]
    fn returns_true_if_entity_has_abuse_role() {
        let entity = build_entity(Some(vec![Technical, Abuse, Administrative]));

        assert!(is_abuse_entity(&entity));
    }

    fn build_entity(roles: Option<Vec<Role>>) -> parser::Entity {
        parser::Entity {
            roles,
            vcard_array: None,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }
}

fn is_abuse_entity(entity: &parser::Entity) -> bool {
    if let Some(roles) = &entity.roles {
        roles.contains(&parser::Role::Abuse)
    } else {
        false
    }
}

#[cfg(test)]
mod has_email_address_tests {
    use super::*;
    
    #[test]
    fn it_returns_false_if_no_vcard_array() {
        assert!(!has_email_address(&entity_without_vcard_array()));
    }

    #[test]
    fn it_returns_false_if_vcard_array_is_empty() {
        assert!(!has_email_address(&entity_with_empty_vcard_array()));
    }

    #[test]
    fn it_returns_false_if_vcard_array_has_no_email_cards() {
        assert!(!has_email_address(&entity_with_no_email_vcards()));
    }

    #[test]
    fn it_returns_false_if_email_cards_have_no_values() {
        assert!(!has_email_address(&entity_with_email_vcards_no_values()));
    }

    #[test]
    fn it_returns_false_if_email_card_values_are_empty_strings() {
        assert!(!has_email_address(&entity_with_emails_vcards_empty_strings()));
    }

    #[test]
    fn it_returns_true_if_email_vcard() {
        assert!(has_email_address(&entity_with_email_vcard()))
    }


    fn entity_without_vcard_array() -> parser::Entity {
        build_entity(None)
    }

    fn entity_with_empty_vcard_array() -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![]
        );

        build_entity(Some(vcard_array))
    }

    fn entity_with_no_email_vcards() -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![
                build_jcard_item("foo", "bar"),
                build_jcard_item("baz", "biz"),
            ]
        );

        build_entity(Some(vcard_array))
    }

    fn entity_with_email_vcards_no_values() -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![
                build_jcard_item_with_no_values("email"),
            ]
        );

        build_entity(Some(vcard_array))
    }

    fn entity_with_emails_vcards_empty_strings() -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![
                build_jcard_item("email", ""),
            ]
        );

        build_entity(Some(vcard_array))
    }

    fn entity_with_email_vcard() -> parser::Entity {
        let vcard_array = parser::JCard(
            parser::JCardType::Vcard,
            vec![
                build_jcard_item("foo", "bar"),
                build_jcard_item("email", "abuse@test.zzz"),
                build_jcard_item("baz", "biz"),
            ]
        );

        build_entity(Some(vcard_array))
    }

    fn build_entity(vcard_array: Option<parser::JCard>) -> parser::Entity {
        parser::Entity {
            roles: None,
            vcard_array,
            handle: None,
            as_event_actor: None,
            public_ids: None,
            entities: None,
            remarks: None,
            links: None,
            events: None,
            status: None,
            port43: None,
            lang: None,
            object_class_name: "entity".into()
        }
    }

    fn build_jcard_item(property_name: &str, value: &str) -> parser::JCardItem {
        use serde_json::map::Map;
        use serde_json::json;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: vec![json!(value)]

        }
    }

    fn build_jcard_item_with_no_values(property_name: &str) -> parser::JCardItem {
        use serde_json::map::Map;

        parser::JCardItem {
            property_name: property_name.into(),
            parameters: Map::new(),
            type_identifier: parser::JCardItemDataType::Text,
            values: vec![]
        }
    }
}

fn has_email_address(entity: &parser::Entity) -> bool {
    if let Some(vcard_array) = &entity.vcard_array {
        let email_values: Vec<&str> = vcard_array
            .items()
            .iter()
            .filter(|item| item.property_name == "email")
            .flat_map(|item| {
                item
                    .values
                    .iter()
                    .flat_map(|value| value.as_str())
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<&str>>()
            })
            .collect();

        !email_values.is_empty()
    } else {
        false
    }
}

#[cfg(test)]
mod test_support {
    use super::*;

    use crate::mountebank::{
        setup_dns_server,
        DnsServerConfig,
    };

    pub fn setup_impostors() {
        setup_dns_server(
            vec![
                DnsServerConfig::response_200(
                    "fake.net",
                    None,
                    "Reg One",
                    "abuse@regone.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                ),
                DnsServerConfig::response_200(
                    "possiblynotfake.com",
                    None,
                    "Reg Two",
                    "abuse@regtwo.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
                ),
                DnsServerConfig::response_200(
                    "morethanlikelyfake.net",
                    None,
                    "Reg Three",
                    "abuse@regthree.zzz",
                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()
                ),
            ]
        );
    }

    pub async fn get_bootstrap() -> Bootstrap {
        let mut client = Client::new();

        client.set_base_bootstrap_url("http://localhost:4545");

        client.fetch_bootstrap().await.unwrap()
    }
}
