use chrono::prelude::*;
use crate:: data::{
    Domain,
    DomainCategory,
    EmailAddressData,
    FulfillmentNode,
    Node,
    OutputData,
    ParsedMail,
    Registrar,
    EmailAddresses
};
use rdap_client::bootstrap::Bootstrap;
use rdap_client::Client;
use rdap_client::parser;
use std::sync::Arc;

#[cfg(test)]
mod populate_tests {
    use super::*;
    use crate:: data::{Domain, EmailAddressData, ParsedMail, Registrar, EmailAddresses};
    use crate::mountebank::*;

    #[test]
    fn populates_output_object_with_domain_data() {
        clear_all_impostors();
        setup_bootstrap_server();
        setup_impostors();
        let bootstrap = tokio_test::block_on(get_bootstrap());

        let input = input_data();
        let expected = output_data();

        let actual = tokio_test::block_on(populate(bootstrap, input));

        assert_eq!(expected, actual);
    }

    fn input_data() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
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

    fn output_data() -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                fulfillment_nodes: vec![
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
                subject: Some("Does not matter".into()),
                email_addresses: EmailAddresses {
                    from: vec![
                        EmailAddressData {
                            address: "someone@fake.net".into(),
                            domain: domain_object(
                                "fake.net",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                            ),
                            registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                        }
                    ],
                    links: vec![
                        EmailAddressData {
                            address: "perp@alsofake.net".into(),
                            domain: domain_object(
                                "alsofake.net",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 15).unwrap()),
                            ),
                            registrar: registrar_object("Reg Four", Some("abuse@regfour.zzz")),
                        }
                    ],
                    reply_to: vec![
                        EmailAddressData {
                            address: "anyone@possiblynotfake.com".into(),
                            domain: domain_object(
                                "possiblynotfake.com",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()),
                            ),
                            registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
                        },
                    ],
                    return_path: vec![
                        EmailAddressData {
                            address: "everyone@morethanlikelyfake.net".into(),
                            domain: domain_object(
                                "morethanlikelyfake.net",
                                Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                            ),
                            registrar: registrar_object("Reg Three", Some("abuse@regthree.zzz")),
                        },
                    ],
                }
            },
            raw_mail: "raw mail text goes here".into(),
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
            ]
        );
    }

    async fn get_bootstrap() -> Bootstrap {
        let mut client = Client::new();

        client.set_base_bootstrap_url("http://localhost:4545");

        client.fetch_bootstrap().await.unwrap()
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

    let (from, links, reply_to, return_path, fulfillment_nodes) = tokio::join!(
        update_from, update_links, update_reply_to, update_return_path, update_fulfillment_nodes
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
            if let Some(response) = get_rdap_data(bootstrap, &domain.name).await {
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
        if let Some(response) = get_rdap_data(bootstrap, name).await {
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

        assert_handle(Arc::clone(&bootstrap), "foo.bar.baz.biz.net", "DOM-BIZ");
        assert_handle(Arc::clone(&bootstrap), "foo.bar.baz.buzz.net", "DOM-BUZZ");
        assert_handle(Arc::clone(&bootstrap), "foo.bar.baz.boz.net", "DOM-BOZ");
        assert_none(Arc::clone(&bootstrap), "un.ob.tai.nium.net");
    }

    #[test]
    fn returns_none_if_no_server() {
        clear_all_impostors();
        setup_bootstrap_server();
        let bootstrap = Arc::new(tokio_test::block_on(get_bootstrap()));

        assert_none(Arc::clone(&bootstrap), "no_server.zzz")
    }

    fn assert_handle(bootstrap: Arc<Bootstrap>, domain_name: &str, expected_handle: &str) {
        let domain = tokio_test::block_on(
            get_rdap_data(Arc::clone(&bootstrap), domain_name)
        ).unwrap();

        assert_eq!(String::from(expected_handle), domain.handle.unwrap())
    }

    fn assert_none(bootstrap: Arc<Bootstrap>, domain_name: &str) {
        let result = tokio_test::block_on(
            get_rdap_data(Arc::clone(&bootstrap), domain_name)
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

async fn get_rdap_data(bootstrap: Arc<Bootstrap>, domain_name: &str) -> Option<parser::Domain> {
    let mut domain_response: Option<parser::Domain> = None;

    if let Some(servers) = bootstrap.dns.find(domain_name) {
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

    domain_response
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
    fn returns_abuse_email() {
        let entities = &[
            non_registrar_entity(),
            registrar_entity(),
        ];

        assert_eq!(
            Some(String::from("abuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    #[test]
    fn returns_none_if_no_registrar_entity() {
        let entities = &[
            non_registrar_entity(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_none_if_registrar_has_none_entities() {
        let entities = &[
            registrar_entity_none_entities(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_none_if_registrar_has_no_abuse_entity() {
        let entities = &[
            registrar_entity_no_abuse_entities(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn returns_last_abuse_entity_details_if_multiple() {
        let entities = &[
            registrar_entity_multiple_abuse_entities(),
        ];

        assert_eq!(
            Some(String::from("alsoabuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    #[test]
    fn registrar_abuse_entity_has_none_vcards() {
        let entities = &[
            registrar_entity_abuse_none_vcards(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn registrar_abuse_entity_no_email_vcard() {
        let entities = &[
            registrar_entity_no_abuse_email(),
        ];

        assert!(extract_abuse_email(entities).is_none());
    }

    #[test]
    fn registrar_abuse_entity_multiple_email_values_returns_last_value() {
        let entities = &[
            registrar_entity_abuse_multiple_email_values(),
        ];

        assert_eq!(
            Some(String::from("alsoabuse@test.zzz")),
            extract_abuse_email(entities)
        )
    }

    #[test]
    fn registrar_abuse_entity_email_is_empty_string_returns_none() {
        let entities = &[
            registrar_abuse_entity_email_is_empty_string(),
        ];

        assert_eq!(
            None,
            extract_abuse_email(entities)
        );
    }

    fn registrar_entity() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Administrative,
                        parser::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        parser::Role::Administrative,
                        parser::Role::Abuse,
                        parser::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        parser::Role::Administrative,
                        parser::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["alsonotabuse@test.zzz"])])
                ),
            ]),
           None
        )
    }

    fn non_registrar_entity() -> parser::Entity {
        build_entity(
            &[parser::Role::Sponsor],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["notregabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_none_entities() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            None,
            None
        )
    }

    fn registrar_entity_no_abuse_entities() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Administrative,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_multiple_abuse_entities() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Administrative,
                        parser::Role::Technical,
                    ],
                    None,
                    Some(&[("email", &["notabuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz"])])
                ),
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["alsoabuse@test.zzz"])]),
                ),
            ]),
            None
        )
    }

    fn registrar_entity_abuse_none_vcards() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    None
                ),
            ]),
            None
        )
    }

    fn registrar_entity_no_abuse_email() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[])
                ),
            ]),
            None
        )
    }

    fn registrar_entity_abuse_multiple_email_values() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &["abuse@test.zzz", "alsoabuse@test.zzz"])])
                ),
            ]),
            None
        )
    }

    fn registrar_abuse_entity_email_is_empty_string() -> parser::Entity {
        build_entity(
            &[parser::Role::Registrar],
            Some(vec![
                build_entity(
                    &[
                        parser::Role::Abuse,
                    ],
                    None,
                    Some(&[("email", &[""])])
                ),
            ]),
            None
        )
    }

    fn build_entity(
        roles: &[parser::Role],
        entities: Option<Vec<parser::Entity>>,
        additional_items: Option<&[(&str, &[&str])]>
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
}

fn extract_abuse_email(entities: &[parser::Entity]) -> Option<String> {
    if let Some(registrar_entity) = find_registrar_entity(entities) {
        if let Some(r_entities) = &registrar_entity.entities {
            let abuse_entities: Vec<&parser::Entity> = r_entities
                .iter()
                .filter(|e| {
                    if let Some(roles) = &e.roles {
                        roles.contains(&parser::Role::Abuse)
                    } else {
                        false
                    }
                })
            .collect();

            if let Some(abuse_entity) = abuse_entities.last() {
                if let Some(vcards) = &abuse_entity.vcard_array {
                    extract_value_from_last_vcard(
                        vcards.items_by_name("email").last()
                    )
                } else {
                    None
                }
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

fn extract_value_from_last_vcard(item_option: Option<&&parser::JCardItem>) -> Option<String> {
    //TODO Get rid of these unwraps

    if let Some(item) = item_option {
        let last_value = item
            .values
            .last()
            .unwrap()
            .as_str()
            .unwrap();

        if !last_value.is_empty() {
            Some(last_value.into())
        } else {
            None
        }
    } else {
        None
    }
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
