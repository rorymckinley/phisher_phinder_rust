use crate::data::{
    Domain,
    EmailAddressData,
    FulfillmentNode,
    HostNode,
    Node,
    OutputData,
    Registrar
};
use crate::result::AppResult;

use prettytable::{Cell, Row, Table};
use regex::Regex;

#[cfg(test)]
mod display_sender_addresses_extended_tests {
    use super::*;
    use crate::data::{
        Domain,
        DomainCategory,
        EmailAddressData,
        ParsedMail,
        Registrar,
        EmailAddresses
    };
    use chrono::prelude::*;

    #[test]
    fn displays_extended_data_for_sender_addresses() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                delivery_nodes: vec![],
                fulfillment_nodes: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: EmailAddresses {
                    from: vec![
                        EmailAddressData {
                            address: "fr@test.www".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.www".into(),
                                    registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regone.zzz".into()),
                                    name: Some("Reg One".into()),
                                }
                            ),
                        }
                    ],
                    reply_to: vec![
                        EmailAddressData {
                            address: "rt@test.xxx".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.xxx".into(),
                                    registration_date: Some(datetime(2022, 12, 2, 3, 4, 5)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regtwo.zzz".into()),
                                    name: Some("Reg Two".into()),
                                }
                            ),
                        },
                        EmailAddressData {
                            address: "rt@test.yyy".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.yyy".into(),
                                    registration_date: Some(datetime(2022, 12, 2, 3, 4, 6)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                                    name: Some("Reg Three".into()),
                                }
                            ),
                        },
                    ],
                    return_path: vec![
                        EmailAddressData {
                            address: "rp@test.zzz".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.zzz".into(),
                                    registration_date: Some(datetime(2022, 12, 3, 4, 5, 7)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regfour.zzz".into()),
                                    name: Some("Reg Four".into()),
                                }
                            ),
                        }
                    ],
                    links: vec![
                        EmailAddressData {
                            address: "l1@test.aaa".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.aaa".into(),
                                    registration_date: Some(datetime(2022, 12, 4, 5, 6, 8)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regfive.zzz".into()),
                                    name: Some("Reg Five".into()),
                                }
                            ),
                        },
                        EmailAddressData {
                            address: "l2@test.bbb".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.bbb".into(),
                                    registration_date: Some(datetime(2022, 12, 4, 5, 6, 9)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regsix.zzz".into()),
                                    name: Some("Reg Six".into()),
                                }
                            ),
                        },
                    ],
                }
            },
            raw_mail: "".into()
        };

        assert_eq!(
            String::from("\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Address Source | Address     | Category | Registrar | Registration Date   | Abuse Email Address |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | From           | fr@test.www | Other    | Reg One   | 2022-12-01 02:03:04 | abuse@regone.zzz    |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Reply-To       | rt@test.xxx | Other    | Reg Two   | 2022-12-02 03:04:05 | abuse@regtwo.zzz    |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            |                | rt@test.yyy | Other    | Reg Three | 2022-12-02 03:04:06 | abuse@regthree.zzz  |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Return-Path    | rp@test.zzz | Other    | Reg Four  | 2022-12-03 04:05:07 | abuse@regfour.zzz   |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Links          | l1@test.aaa | Other    | Reg Five  | 2022-12-04 05:06:08 | abuse@regfive.zzz   |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            |                | l2@test.bbb | Other    | Reg Six   | 2022-12-04 05:06:09 | abuse@regsix.zzz    |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            "),
            display_sender_addresses_extended(&data).unwrap()
        )
    }

    #[test]
    fn display_extended_data_no_domain_data() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                delivery_nodes: vec![],
                fulfillment_nodes: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: EmailAddresses {
                    from: vec![
                        EmailAddressData {
                            address: "fr@test.xxx".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.xxx".into(),
                                    registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regone.zzz".into()),
                                    name: Some("Reg One".into()),
                                }
                            ),
                        }
                    ],
                    reply_to: vec![
                        EmailAddressData {
                            address: "rt@test.yyy".into(),
                            domain: None,
                            registrar: None,
                        }
                    ],
                    return_path: vec![
                        EmailAddressData {
                            address: "rp@test.zzz".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.zzz".into(),
                                    registration_date: Some(datetime(2022, 12, 3, 4, 5, 6)),
                                    abuse_email_address: None,
                                }
                            ),
                            registrar: Some(
                                Registrar {
                                    abuse_email_address: Some("abuse@regthree.zzz".into()),
                                    name: Some("Reg Three".into()),
                                }
                            ),
                        }
                    ],
                    links: vec![],
                }
            },
            raw_mail: "".into()
        };

        assert_eq!(
            String::from("\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Address Source | Address     | Category | Registrar | Registration Date   | Abuse Email Address |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | From           | fr@test.xxx | Other    | Reg One   | 2022-12-01 02:03:04 | abuse@regone.zzz    |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Reply-To       | rt@test.yyy | N/A      | N/A       | N/A                 | N/A                 |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Return-Path    | rp@test.zzz | Other    | Reg Three | 2022-12-03 04:05:06 | abuse@regthree.zzz  |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            "),
            display_sender_addresses_extended(&data).unwrap()
        )
    }

    fn datetime(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> chrono::DateTime<Utc> {
        chrono::Utc.with_ymd_and_hms(y, m, d, h, min, s).single().unwrap()
    }
}

pub fn display_sender_addresses_extended(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![
            Cell::new("Address Source"),
            Cell::new("Address"),
            Cell::new("Category"),
            Cell::new("Registrar"),
            Cell::new("Registration Date"),
            Cell::new("Abuse Email Address"),
        ])
    );

    let addresses = &data.parsed_mail.email_addresses;

    sender_address_row(&mut table, "From", &addresses.from);
    sender_address_row(&mut table, "Reply-To", &addresses.reply_to);
    sender_address_row(&mut table, "Return-Path", &addresses.return_path);
    sender_address_row(&mut table, "Links", &addresses.links);

    table_to_string(&table)
}

fn table_to_string(table: &Table) -> AppResult<String> {
    let mut buffer: Vec<u8> = Vec::new();

    table.print(&mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

fn sender_address_row(
    table: &mut Table, label: &str, email_address_data: &[EmailAddressData]
) {
    for (
        pos,
        EmailAddressData {address, domain, registrar}
    ) in email_address_data.iter().enumerate() {
        let actual_label = if pos == 0 {
            label
        } else {
            ""
        };

        table.add_row(
            Row::new(
                vec![
                    Cell::new(actual_label),
                    Cell::new(address),
                    domain_category_cell(domain.as_ref()),
                    registrar_name_cell(registrar.as_ref()),
                    registration_date_cell(domain.as_ref()),
                    registrar_abuse_email_cell(registrar.as_ref())
                ]
            )
        );
    }
}

#[cfg(test)]
mod display_fulfillment_nodes_tests {
    use super::*;
    use crate::data::{DomainCategory, FulfillmentNode, ParsedMail, EmailAddresses};
    use chrono::prelude::*;

    #[test]
    fn display_fulfillment_nodes_details_no_registrar_data() {
        let node_bar = fulfillment_node_with_rdap_data();
        let mut node_baz = FulfillmentNode::new("https://foo.baz");
        node_baz.set_hidden("https://redirect.baz");
        let node_biz = FulfillmentNode::new("https://foo.biz");

        let data = OutputData {
            parsed_mail: ParsedMail {
                delivery_nodes: vec![],
                fulfillment_nodes: vec![node_bar, node_baz, node_biz],
                subject: None,
                email_addresses: EmailAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                    links: vec![],
                }
            },
            raw_mail: "".into()
        };

        assert_eq!(
            String::from("\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            | Visible                                                                            | Hidden                                                                                  |\n\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            | Url             | Category | Registrar | Registration Date   | Abuse Email Address | Url                  | Category | Registrar | Registration Date   | Abuse Email Address |\n\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            | https://foo.bar | Other    | Reg Two   | 2022-11-18 10:11:15 | abuse@regtwo.zzz    | https://redirect.bar | Other    | Reg One   | 2022-11-18 10:11:14 | abuse@regone.zzz    |\n\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            | https://foo.baz | Other    | N/A       | N/A                 | N/A                 | https://redirect.baz | Other    | N/A       | N/A                 | N/A                 |\n\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            | https://foo.biz | Other    | N/A       | N/A                 | N/A                 | N/A                  | N/A      | N/A       | N/A                 | N/A                 |\n\
            +-----------------+----------+-----------+---------------------+---------------------+----------------------+----------+-----------+---------------------+---------------------+\n\
            "),
            display_fulfillment_nodes(&data).unwrap()
        )
    }

    fn fulfillment_node_with_rdap_data() -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(
                Node {
                    domain: domain_object(
                        "redirect.bar",
                        Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                    ),
                    registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                    url: "https://redirect.bar".into(),
                }
            ),
            visible: Node {
                domain: domain_object(
                    "foo.bar",
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 15).unwrap()),
                ),
                registrar: registrar_object("Reg Two", Some("abuse@regtwo.zzz")),
                url: "https://foo.bar".into(),
            },
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

}

pub fn display_fulfillment_nodes(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![
            Cell::new("Visible").with_hspan(5),
            Cell::new("Hidden").with_hspan(5),
        ]),
    );
    table.add_row(
        Row::new(vec![
            Cell::new("Url"),
            Cell::new("Category"),
            Cell::new("Registrar"),
            Cell::new("Registration Date"),
            Cell::new("Abuse Email Address"),
            Cell::new("Url"),
            Cell::new("Category"),
            Cell::new("Registrar"),
            Cell::new("Registration Date"),
            Cell::new("Abuse Email Address"),
        ])
    );

    for node in data.parsed_mail.fulfillment_nodes.iter() {
        fulfillment_node_row(&mut table, node);
    }

    table_to_string(&table)
}

fn fulfillment_node_row(table: &mut Table, node: &FulfillmentNode) {
    let hidden_url = if let Some(url) = node.hidden_url() {
        url
    } else {
        "N/A".into()
    };

    let hidden_domain = if let Some(Node { domain: Some(domain), .. }) = &node.hidden {
        Some(domain)
    } else {
        None
    };

    let hidden_registrar = if let Some(Node { registrar: Some(registrar), .. }) = &node.hidden {
        Some(registrar)
    } else {
        None
    };

    table.add_row(
        Row::new(vec![
            url_cell(node.visible_url()),
            domain_category_cell(node.visible.domain.as_ref()),
            registrar_name_cell(node.visible.registrar.as_ref()),
            registration_date_cell(node.visible.domain.as_ref()),
            registrar_abuse_email_cell(node.visible.registrar.as_ref()),
            url_cell(&hidden_url),
            domain_category_cell(hidden_domain),
            registrar_name_cell(hidden_registrar),
            registration_date_cell(hidden_domain),
            registrar_abuse_email_cell(hidden_registrar),
        ])
    );
}

#[cfg(test)]
mod domain_category_cell_tests {
    use super::*;
    use crate::data::DomainCategory;

    #[test]
    fn returns_n_a_cell_if_domain_is_none() {
        assert_eq!(Cell::new("N/A"), domain_category_cell(None))
    }

    #[test]
    fn returns_the_category_if_domain_exists() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: None
        };

        assert_eq!(Cell::new("Other"), domain_category_cell(Some(&domain)));
    }
}

fn domain_category_cell(domain_opt: Option<&Domain>) -> Cell {
    if let Some(domain) = domain_opt {
        Cell::new(&domain.category.to_string())
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod registration_date_cell_tests {
    use super::*;
    use chrono::prelude::*;
    use crate::data::DomainCategory;

    #[test]
    fn returns_n_a_cell_if_domain_is_none() {
        assert_eq!(Cell::new("N/A"), registration_date_cell(None))
    }

    #[test]
    fn returns_date_if_domain_has_registration_date() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: Some(datetime(2022, 12, 25, 10, 11, 12))
        };

        assert_eq!(Cell::new("2022-12-25 10:11:12"), registration_date_cell(Some(&domain)));
    }

    #[test]
    fn returns_n_a_if_no_date() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: None,
        };

        assert_eq!(Cell::new("N/A"), registration_date_cell(Some(&domain)));
    }

    fn datetime(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> chrono::DateTime<Utc> {
        chrono::Utc.with_ymd_and_hms(y, m, d, h, min, s).single().unwrap()
    }
}

fn registration_date_cell(domain_opt: Option<&Domain>) -> Cell {
    if let Some(Domain { registration_date: Some(registration_date), .. }) = domain_opt {
        Cell::new(&registration_date.format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod registrar_name_cell_tests {
    use super::*;

    #[test]
    fn returns_n_a_if_registrar_none() {
        assert_eq!(Cell::new("N/A"), registrar_name_cell(None))
    }

    #[test]
    fn returns_registrar_name_if_registrar() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: Some("Reg One".into()),
        };

        assert_eq!(Cell::new("Reg One"), registrar_name_cell(Some(&registrar)));
    }

    #[test]
    fn returns_n_a_if_no_name() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(Cell::new("N/A"), registrar_name_cell(Some(&registrar)));
    }
}

fn registrar_name_cell(registrar_opt: Option<&Registrar>) -> Cell {
    if let Some(Registrar {name: Some(name), ..}) = registrar_opt {
        Cell::new(name)
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod registrar_abuse_email_cell_tests {
    use super::*;

    #[test]
    fn returns_n_a_if_registrar_none() {
        assert_eq!(Cell::new("N/A"), registrar_abuse_email_cell(None));
    }

    #[test]
    fn returns_email_address() {
        let registrar = Registrar {
            abuse_email_address: Some("abuse@regone.co.za".into()),
            name: None,
        };

        assert_eq!(Cell::new("abuse@regone.co.za"), registrar_abuse_email_cell(Some(&registrar)));
    }

    #[test]
    fn returns_n_a_if_abuse_email_address_none() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(Cell::new("N/A"), registrar_abuse_email_cell(Some(&registrar)));
    }
}

fn registrar_abuse_email_cell(registrar_opt: Option<&Registrar>) -> Cell {
    if let Some(Registrar {abuse_email_address: Some(abuse_email_address), ..}) = registrar_opt {
        Cell::new(abuse_email_address)
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod display_url_tests {
    use super::*;

    #[test]
    fn it_displays_the_url() {
        assert_eq!(
            Cell::new("https://foo.bar"),
            url_cell("https://foo.bar"),
        )
    }

    #[test]
    fn it_trims_query_params_when_displaying() {
        assert_eq!(
            Cell::new("https://foo.bar?..."),
            url_cell("https://foo.bar?baz=buzz")
        )
    }
}

fn url_cell(url: &str) -> Cell {
    let re = Regex::new(r"\?.+\z").unwrap();

    Cell::new(&re.replace_all(url, "?..."))
}

#[cfg(test)]
mod display_delivery_nodes_tests {
    use crate::data::{DeliveryNode, EmailAddresses, HostNode, ParsedMail};
    use super::*;

    #[test]
    fn displays_delivery_nodes() {
        let data = build_output_data(vec![
            delivery_node(
                Some("a.bar.com"),
                Some("b.bar.com"),
                Some("10.10.10.10"),
                Some("a.foo.com"),
            ),
            delivery_node(None, Some("b.baz.com"), None, None),
        ]);

        assert_eq!(
            String::from("\
            +-----------------+---------------+-------------+-----------+\n\
            | Advertised host | Observed host | Observed IP | Recipient |\n\
            +-----------------+---------------+-------------+-----------+\n\
            | a.bar.com       | b.bar.com     | 10.10.10.10 | a.foo.com |\n\
            +-----------------+---------------+-------------+-----------+\n\
            | N/A             | b.baz.com     | N/A         | N/A       |\n\
            +-----------------+---------------+-------------+-----------+\n\
            "),
            display_delivery_nodes(&data).unwrap()
        )
    }

    fn build_output_data(delivery_nodes: Vec<DeliveryNode>) -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                delivery_nodes,
                fulfillment_nodes: vec![],
                subject: None,
                email_addresses: EmailAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                    links: vec![],
                }
            },
            raw_mail: "".into()
        }
    }

    fn delivery_node(
        advertised_host: Option<&str>,
        observed_host: Option<&str>,
        observed_ip: Option<&str>,
        recipient: Option<&str>
    ) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: host_node(advertised_host, None),
            observed_sender: host_node(observed_host, observed_ip),
            recipient: recipient.map(String::from),
            time: None
        }
    }

    fn host_node(host: Option<&str>, ip: Option<&str>) -> Option<HostNode> {
        HostNode::new(host, ip).ok()
    }
}

pub fn display_delivery_nodes(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![
            Cell::new("Advertised host"),
            Cell::new("Observed host"),
            Cell::new("Observed IP"),
            Cell::new("Recipient")
        ]),
    );

    for node in data.parsed_mail.delivery_nodes.iter() {
        table.add_row(
            Row::new(
                vec![
                    Cell::new(&display_host(node.advertised_sender.as_ref())),
                    Cell::new(&display_host(node.observed_sender.as_ref())),
                    Cell::new(&display_ip(node.observed_sender.as_ref())),
                    Cell::new(&display_recipient(node.recipient.as_ref())),
                ]
            )
        );
    }

    table_to_string(&table)
}

#[cfg(test)]
mod display_host_tests {
    use crate::data::HostNode;
    use super::*;

    #[test]
    fn returns_the_host() {
        let node = HostNode::new(Some("foo"), None).unwrap();

        assert_eq!("foo", display_host(Some(&node)));
    }

    #[test]
    fn returns_na_if_no_host() {
        let node = HostNode::new(None, Some("10.10.10.10")).unwrap();

        assert_eq!("N/A", display_host(Some(&node)));
    }

    #[test]
    fn returns_na_if_no_node() {
        assert_eq!("N/A", display_host(None));
    }
}

fn display_host(node_option: Option<&HostNode>) -> String {
    if let Some(HostNode { host: Some(host_val), .. }) = node_option {
        String::from(host_val)
    } else {
        String::from("N/A")
    }
}

#[cfg(test)]
mod display_ip_tests {
    use crate::data::HostNode;
    use super::*;

    #[test]
    fn returns_the_ip() {
        let node = HostNode::new(Some("foo"), Some("10.10.10.10")).unwrap();

        assert_eq!("10.10.10.10", display_ip(Some(&node)));
    }

    #[test]
    fn returns_na_if_no_ip() {
        let node = HostNode::new(Some("foo"), None).unwrap();

        assert_eq!("N/A", display_ip(Some(&node)));
    }

    #[test]
    fn returns_na_if_no_node() {
        assert_eq!("N/A", display_ip(None));
    }
}

fn display_ip(node_option: Option<&HostNode>) -> String {
    if let Some(HostNode { ip_address: Some(ip_val), .. }) = node_option {
        String::from(ip_val)
    } else {
        String::from("N/A")
    }
}

#[cfg(test)]
mod display_recipient_tests {
    use super::*;

    #[test]
    fn returns_the_recipient() {
        assert_eq!("foo", display_recipient(Some(&String::from("foo"))))
    }

    #[test]
    fn returns_na_if_no_recipient() {
        assert_eq!("N/A", display_recipient(None))
    }
}

fn display_recipient(recipient_opt: Option<&String>) -> String {
    if let Some(recipient) = recipient_opt {
        String::from(recipient)
    } else {
        String::from("N/A")
    }
}
