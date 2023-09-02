use chrono::prelude::*;
use crate::authentication_results::AuthenticationResults;
use crate::data::{
    DeliveryNode,
    Domain,
    EmailAddressData,
    FulfillmentNode,
    HostNode,
    InfrastructureProvider,
    Node,
    OutputData,
    Registrar,
};
use crate::result::AppResult;
use crate::run::Run;

use prettytable::{Cell, Row, Table};
use regex::Regex;

#[cfg(test)]
mod display_sender_addresses_extended_tests {
    use super::*;
    use crate::authentication_results::{Dkim, DkimResult, Spf, SpfResult};
    use crate::data::{
        Domain, DomainCategory, EmailAddressData, EmailAddresses, ParsedMail, Registrar,
    };
    use crate::message_source::MessageSource;

    #[test]
    fn displays_extended_data_for_sender_addresses() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                authentication_results: authentication_results(),
                delivery_nodes: vec![],
                fulfillment_nodes: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: EmailAddresses {
                    from: vec![EmailAddressData {
                        address: "fr@test.www".into(),
                        domain: Some(Domain {
                            category: DomainCategory::Other,
                            name: "test.www".into(),
                            registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                            abuse_email_address: None,
                        }),
                        registrar: Some(Registrar {
                            abuse_email_address: Some("abuse@regone.zzz".into()),
                            name: Some("Reg One".into()),
                        }),
                    }],
                    reply_to: vec![
                        EmailAddressData {
                            address: "rt@test.xxx".into(),
                            domain: Some(Domain {
                                category: DomainCategory::Other,
                                name: "test.xxx".into(),
                                registration_date: Some(datetime(2022, 12, 2, 3, 4, 5)),
                                abuse_email_address: None,
                            }),
                            registrar: Some(Registrar {
                                abuse_email_address: Some("abuse@regtwo.zzz".into()),
                                name: Some("Reg Two".into()),
                            }),
                        },
                        EmailAddressData {
                            address: "rt@test.yyy".into(),
                            domain: Some(Domain {
                                category: DomainCategory::Other,
                                name: "test.yyy".into(),
                                registration_date: Some(datetime(2022, 12, 2, 3, 4, 6)),
                                abuse_email_address: None,
                            }),
                            registrar: Some(Registrar {
                                abuse_email_address: Some("abuse@regthree.zzz".into()),
                                name: Some("Reg Three".into()),
                            }),
                        },
                    ],
                    return_path: vec![EmailAddressData {
                        address: "rp@test.zzz".into(),
                        domain: Some(Domain {
                            category: DomainCategory::Other,
                            name: "test.zzz".into(),
                            registration_date: Some(datetime(2022, 12, 3, 4, 5, 7)),
                            abuse_email_address: None,
                        }),
                        registrar: Some(Registrar {
                            abuse_email_address: Some("abuse@regfour.zzz".into()),
                            name: Some("Reg Four".into()),
                        }),
                    }],
                    links: vec![
                        EmailAddressData {
                            address: "l1@test.aaa".into(),
                            domain: Some(Domain {
                                category: DomainCategory::Other,
                                name: "test.aaa".into(),
                                registration_date: Some(datetime(2022, 12, 4, 5, 6, 8)),
                                abuse_email_address: None,
                            }),
                            registrar: Some(Registrar {
                                abuse_email_address: Some("abuse@regfive.zzz".into()),
                                name: Some("Reg Five".into()),
                            }),
                        },
                        EmailAddressData {
                            address: "l2@test.bbb".into(),
                            domain: Some(Domain {
                                category: DomainCategory::Other,
                                name: "test.bbb".into(),
                                registration_date: Some(datetime(2022, 12, 4, 5, 6, 9)),
                                abuse_email_address: None,
                            }),
                            registrar: Some(Registrar {
                                abuse_email_address: Some("abuse@regsix.zzz".into()),
                                name: Some("Reg Six".into()),
                            }),
                        },
                    ],
                },
            },
            message_source: MessageSource::new(""),
            reportable_entities: None,
            run_id: None,
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
                authentication_results: authentication_results(),
                delivery_nodes: vec![],
                fulfillment_nodes: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: EmailAddresses {
                    from: vec![EmailAddressData {
                        address: "fr@test.xxx".into(),
                        domain: Some(Domain {
                            category: DomainCategory::Other,
                            name: "test.xxx".into(),
                            registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                            abuse_email_address: None,
                        }),
                        registrar: Some(Registrar {
                            abuse_email_address: Some("abuse@regone.zzz".into()),
                            name: Some("Reg One".into()),
                        }),
                    }],
                    reply_to: vec![EmailAddressData {
                        address: "rt@test.yyy".into(),
                        domain: None,
                        registrar: None,
                    }],
                    return_path: vec![EmailAddressData {
                        address: "rp@test.zzz".into(),
                        domain: Some(Domain {
                            category: DomainCategory::Other,
                            name: "test.zzz".into(),
                            registration_date: Some(datetime(2022, 12, 3, 4, 5, 6)),
                            abuse_email_address: None,
                        }),
                        registrar: Some(Registrar {
                            abuse_email_address: Some("abuse@regthree.zzz".into()),
                            name: Some("Reg Three".into()),
                        }),
                    }],
                    links: vec![],
                },
            },
            message_source: MessageSource::new(""),
            reportable_entities: None,
            run_id: None,
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
        chrono::Utc
            .with_ymd_and_hms(y, m, d, h, min, s)
            .single()
            .unwrap()
    }

    fn authentication_results() -> Option<AuthenticationResults> {
        Some(AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Fail),
                selector: Some("".into()),
                signature_snippet: Some("".into()),
                user_identifier_snippet: Some("".into()),
            }),
            service_identifier: Some("does.not.matter".into()),
            spf: Some(Spf {
                ip_address: Some("".into()),
                result: Some(SpfResult::SoftFail),
                smtp_mailfrom: Some("".into()),
            }),
        })
    }
}

pub fn display_sender_addresses_extended(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Address Source"),
        Cell::new("Address"),
        Cell::new("Category"),
        Cell::new("Registrar"),
        Cell::new("Registration Date"),
        Cell::new("Abuse Email Address"),
    ]));

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

fn sender_address_row(table: &mut Table, label: &str, email_address_data: &[EmailAddressData]) {
    for (
        pos,
        EmailAddressData {
            address,
            domain,
            registrar,
        },
    ) in email_address_data.iter().enumerate()
    {
        let actual_label = if pos == 0 { label } else { "" };

        table.add_row(Row::new(vec![
            Cell::new(actual_label),
            Cell::new(address),
            domain_category_cell(domain.as_ref()),
            registrar_name_cell(registrar.as_ref()),
            registration_date_cell(domain.as_ref()),
            registrar_abuse_email_cell(registrar.as_ref()),
        ]));
    }
}

#[cfg(test)]
mod display_fulfillment_nodes_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Dkim, DkimResult, Spf, SpfResult};
    use crate::data::{DomainCategory, EmailAddresses, FulfillmentNode, ParsedMail};
    use crate::message_source::MessageSource;

    #[test]
    fn display_fulfillment_nodes_details_no_registrar_data() {
        let node_bar = fulfillment_node_with_rdap_data();
        let mut node_baz = FulfillmentNode::new("https://foo.baz");
        node_baz.set_hidden("https://redirect.baz");
        let node_biz = FulfillmentNode::new("https://foo.biz");

        let data = OutputData {
            parsed_mail: ParsedMail {
                authentication_results: authentication_results(),
                delivery_nodes: vec![],
                fulfillment_nodes: vec![node_bar, node_baz, node_biz],
                subject: None,
                email_addresses: EmailAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                    links: vec![],
                },
            },
            message_source: MessageSource::new(""),
            reportable_entities: None,
            run_id: None,
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
            hidden: Some(Node {
                domain: domain_object(
                    "redirect.bar",
                    Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                ),
                registrar: registrar_object("Reg One", Some("abuse@regone.zzz")),
                url: "https://redirect.bar".into(),
            }),
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

    fn domain_object(name: &str, registration_date: Option<DateTime<Utc>>) -> Option<Domain> {
        Some(Domain {
            category: DomainCategory::Other,
            name: name.into(),
            registration_date,
            abuse_email_address: None,
        })
    }

    fn registrar_object(name: &str, abuse_email_address: Option<&str>) -> Option<Registrar> {
        Some(Registrar {
            name: Some(name.into()),
            abuse_email_address: abuse_email_address.map(String::from),
        })
    }

    fn authentication_results() -> Option<AuthenticationResults> {
        Some(AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Fail),
                selector: Some("".into()),
                signature_snippet: Some("".into()),
                user_identifier_snippet: Some("".into()),
            }),
            service_identifier: Some("does.not.matter".into()),
            spf: Some(Spf {
                ip_address: Some("".into()),
                result: Some(SpfResult::SoftFail),
                smtp_mailfrom: Some("".into()),
            }),
        })
    }
}

pub fn display_fulfillment_nodes(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Visible").with_hspan(5),
        Cell::new("Hidden").with_hspan(5),
    ]));
    table.add_row(Row::new(vec![
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
    ]));

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

    let hidden_domain = if let Some(Node {
        domain: Some(domain),
        ..
    }) = &node.hidden
    {
        Some(domain)
    } else {
        None
    };

    let hidden_registrar = if let Some(Node {
        registrar: Some(registrar),
        ..
    }) = &node.hidden
    {
        Some(registrar)
    } else {
        None
    };

    table.add_row(Row::new(vec![
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
    ]));
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
            registration_date: None,
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
            registration_date: Some(datetime(2022, 12, 25, 10, 11, 12)),
        };

        assert_eq!(
            Cell::new("2022-12-25 10:11:12"),
            registration_date_cell(Some(&domain))
        );
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
        chrono::Utc
            .with_ymd_and_hms(y, m, d, h, min, s)
            .single()
            .unwrap()
    }
}

fn registration_date_cell(domain_opt: Option<&Domain>) -> Cell {
    if let Some(Domain {
        registration_date: Some(registration_date),
        ..
    }) = domain_opt
    {
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
    if let Some(Registrar {
        name: Some(name), ..
    }) = registrar_opt
    {
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

        assert_eq!(
            Cell::new("abuse@regone.co.za"),
            registrar_abuse_email_cell(Some(&registrar))
        );
    }

    #[test]
    fn returns_n_a_if_abuse_email_address_none() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            registrar_abuse_email_cell(Some(&registrar))
        );
    }
}

fn registrar_abuse_email_cell(registrar_opt: Option<&Registrar>) -> Cell {
    if let Some(Registrar {
        abuse_email_address: Some(abuse_email_address),
        ..
    }) = registrar_opt
    {
        Cell::new(abuse_email_address)
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod ip_provider_name_cell_tests {
    use super::*;

    #[test]
    fn returns_cell_with_na_if_no_infrastructure_provider() {
        assert_eq!(Cell::new("N/A"), ip_provider_name_cell(None))
    }

    #[test]
    fn returns_cell_with_na_if_no_name() {
        let provider = InfrastructureProvider {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(Cell::new("N/A"), ip_provider_name_cell(Some(&provider)))
    }

    #[test]
    fn returns_cell_with_provider_name() {
        let provider = InfrastructureProvider {
            abuse_email_address: None,
            name: Some("Acme".into()),
        };

        assert_eq!(Cell::new("Acme"), ip_provider_name_cell(Some(&provider)))
    }
}

fn ip_provider_name_cell(provider_opt: Option<&InfrastructureProvider>) -> Cell {
    if let Some(provider) = &provider_opt {
        if let Some(name) = &provider.name {
            Cell::new(name)
        } else {
            Cell::new("N/A")
        }
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod ip_provider_abuse_email_cell_tests {
    use super::*;

    #[test]
    fn returns_na_cell_if_no_provider() {
        assert_eq!(Cell::new("N/A"), ip_provider_abuse_email_cell(None));
    }

    #[test]
    fn returns_na_cell_if_no_abuse_email() {
        let provider = InfrastructureProvider {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            ip_provider_abuse_email_cell(Some(&provider))
        );
    }

    #[test]
    fn returns_cell_with_name() {
        let provider = InfrastructureProvider {
            abuse_email_address: Some("abuse@acme.zzz".into()),
            name: None,
        };

        assert_eq!(
            Cell::new("abuse@acme.zzz"),
            ip_provider_abuse_email_cell(Some(&provider))
        );
    }
}

fn ip_provider_abuse_email_cell(provider_opt: Option<&InfrastructureProvider>) -> Cell {
    if let Some(provider) = &provider_opt {
        if let Some(abuse_email_address) = &provider.abuse_email_address {
            Cell::new(abuse_email_address)
        } else {
            Cell::new("N/A")
        }
    } else {
        Cell::new("N/A")
    }
}

#[cfg(test)]
mod display_url_tests {
    use super::*;

    #[test]
    fn it_displays_the_url() {
        assert_eq!(Cell::new("https://foo.bar"), url_cell("https://foo.bar"),)
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
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Dkim, DkimResult, Spf, SpfResult};
    use crate::data::{
        DeliveryNode, DomainCategory, EmailAddresses, HostNode, InfrastructureProvider, ParsedMail,
    };
    use crate::message_source::MessageSource;

    #[test]
    fn displays_delivery_nodes_without_rdap_data() {
        let data = build_output_data(vec![
            delivery_node(
                Some("a.bar.com"),
                Some("b.bar.com"),
                Some("10.10.10.10"),
                0,
                Some("a.foo.com"),
            ),
            trusted_delivery_node(None, Some("b.baz.com"), None, 1, None),
        ]);

        assert_eq!(
            String::from("\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            | Recipient | Advertised | Observed                                                                                                                      | Trusted |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            |           | Host       | Host      | Registrar | Host Registration Date | Registrar Abuse Address | IP          | IP Provider | Provider Abuse Address |         |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            | a.foo.com | a.bar.com  | b.bar.com | N/A       | N/A                    | N/A                     | 10.10.10.10 | N/A         | N/A                    | No      |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            | N/A       | N/A        | b.baz.com | N/A       | N/A                    | N/A                     | N/A         | N/A         | N/A                    | Yes     |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            "),
            display_delivery_nodes(&data).unwrap()
        )
    }

    #[test]
    fn orders_delivery_nodes_by_position() {
        let data = build_output_data(vec![
            delivery_node(None, Some("c.baz.com"), None, 2, None),
            trusted_delivery_node(None, Some("a.baz.com"), None, 0, None),
            delivery_node(None, Some("d.baz.com"), None, 3, None),
            delivery_node(None, Some("b.baz.com"), None, 1, None),
        ]);

        assert_eq!(
            String::from("\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | Recipient | Advertised | Observed                                                                                                              | Trusted |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            |           | Host       | Host      | Registrar | Host Registration Date | Registrar Abuse Address | IP  | IP Provider | Provider Abuse Address |         |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | N/A       | N/A        | a.baz.com | N/A       | N/A                    | N/A                     | N/A | N/A         | N/A                    | Yes     |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | N/A       | N/A        | b.baz.com | N/A       | N/A                    | N/A                     | N/A | N/A         | N/A                    | No      |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | N/A       | N/A        | c.baz.com | N/A       | N/A                    | N/A                     | N/A | N/A         | N/A                    | No      |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | N/A       | N/A        | d.baz.com | N/A       | N/A                    | N/A                     | N/A | N/A         | N/A                    | No      |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            "),
            display_delivery_nodes(&data).unwrap()
        )
    }

    #[test]
    fn displays_delivery_nodes_with_rdap_data() {
        let data = build_output_data(vec![delivery_node_with_rdap_data(
            advertised_sender("a.bar.com"),
            observed_sender(
                "b.bar.com",
                "10.10.10.10",
                registration_date(2022, 11, 18, 10, 11, 15),
                registrar("Acme", "abuse@acme.zzz"),
                provider("HackMe", "abuse@hackme.zzz"),
            ),
            Some("a.foo.com"),
        )]);

        assert_eq!(
            String::from("\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            | Recipient | Advertised | Observed                                                                                                                      | Trusted |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            |           | Host       | Host      | Registrar | Host Registration Date | Registrar Abuse Address | IP          | IP Provider | Provider Abuse Address |         |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            | a.foo.com | a.bar.com  | b.bar.com | Acme      | 2022-11-18 10:11:15    | abuse@acme.zzz          | 10.10.10.10 | HackMe      | abuse@hackme.zzz       | No      |\n\
            +-----------+------------+-----------+-----------+------------------------+-------------------------+-------------+-------------+------------------------+---------+\n\
            "),
            display_delivery_nodes(&data).unwrap()
        )
    }

    #[test]
    fn displays_delivery_nodes_without_observed_sender() {
        let data = build_output_data(vec![delivery_node_with_rdap_data(
            advertised_sender("a.bar.com"),
            None,
            Some("a.foo.com"),
        )]);

        assert_eq!(
            String::from("\
            +-----------+------------+------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | Recipient | Advertised | Observed                                                                                                         | Trusted |\n\
            +-----------+------------+------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            |           | Host       | Host | Registrar | Host Registration Date | Registrar Abuse Address | IP  | IP Provider | Provider Abuse Address |         |\n\
            +-----------+------------+------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            | a.foo.com | a.bar.com  | N/A  | N/A       | N/A                    | N/A                     | N/A | N/A         | N/A                    | No      |\n\
            +-----------+------------+------+-----------+------------------------+-------------------------+-----+-------------+------------------------+---------+\n\
            "),
            display_delivery_nodes(&data).unwrap()
        )
    }

    fn build_output_data(delivery_nodes: Vec<DeliveryNode>) -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                authentication_results: authentication_results(),
                delivery_nodes,
                fulfillment_nodes: vec![],
                subject: None,
                email_addresses: EmailAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                    links: vec![],
                },
            },
            message_source: MessageSource::new(""),
            reportable_entities: None,
            run_id: None,
        }
    }

    fn delivery_node(
        advertised_host: Option<&str>,
        observed_host: Option<&str>,
        observed_ip: Option<&str>,
        position: usize,
        recipient: Option<&str>,
    ) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: host_node(advertised_host, None),
            observed_sender: host_node(observed_host, observed_ip),
            position,
            recipient: recipient.map(String::from),
            time: None,
            trusted: false,
        }
    }

    fn trusted_delivery_node(
        advertised_host: Option<&str>,
        observed_host: Option<&str>,
        observed_ip: Option<&str>,
        position: usize,
        recipient: Option<&str>,
    ) -> DeliveryNode {
        DeliveryNode {
            advertised_sender: host_node(advertised_host, None),
            observed_sender: host_node(observed_host, observed_ip),
            position,
            recipient: recipient.map(String::from),
            time: None,
            trusted: true,
        }
    }

    fn delivery_node_with_rdap_data(
        advertised_sender: Option<HostNode>,
        observed_sender: Option<HostNode>,
        recipient: Option<&str>,
    ) -> DeliveryNode {
        DeliveryNode {
            advertised_sender,
            observed_sender,
            position: 0,
            recipient: recipient.map(String::from),
            time: None,
            trusted: false,
        }
    }

    fn advertised_sender(host: &str) -> Option<HostNode> {
        host_node(Some(host), None)
    }

    fn observed_sender(
        host: &str,
        ip_address: &str,
        registration_date: Option<DateTime<Utc>>,
        registrar: Option<Registrar>,
        infrastructure_provider: Option<InfrastructureProvider>,
    ) -> Option<HostNode> {
        Some(HostNode {
            domain: Some(Domain {
                abuse_email_address: None,
                category: DomainCategory::Other,
                name: host.into(),
                registration_date,
            }),
            host: Some(host.into()),
            ip_address: Some(ip_address.into()),
            registrar,
            infrastructure_provider,
        })
    }

    fn registration_date(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Option<DateTime<Utc>> {
        Some(
            Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
                .unwrap(),
        )
    }

    fn registrar(name: &str, abuse_email_address: &str) -> Option<Registrar> {
        Some(Registrar {
            abuse_email_address: Some(abuse_email_address.into()),
            name: Some(name.into()),
        })
    }

    fn provider(name: &str, abuse_email_address: &str) -> Option<InfrastructureProvider> {
        Some(InfrastructureProvider {
            abuse_email_address: Some(abuse_email_address.into()),
            name: Some(name.into()),
        })
    }

    fn host_node(host: Option<&str>, ip: Option<&str>) -> Option<HostNode> {
        HostNode::new(host, ip).ok()
    }

    fn authentication_results() -> Option<AuthenticationResults> {
        Some(AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Fail),
                selector: Some("".into()),
                signature_snippet: Some("".into()),
                user_identifier_snippet: Some("".into()),
            }),
            service_identifier: Some("does.not.matter".into()),
            spf: Some(Spf {
                ip_address: Some("".into()),
                result: Some(SpfResult::SoftFail),
                smtp_mailfrom: Some("".into()),
            }),
        })
    }
}

pub fn display_delivery_nodes(data: &OutputData) -> AppResult<String> {
    // TODO look for reuse between this and display_fulfillment_nodes
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        Cell::new("Recipient"),
        Cell::new("Advertised"),
        Cell::new("Observed").with_hspan(7),
        Cell::new("Trusted"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new(""),
        Cell::new("Host"),
        Cell::new("Host"),
        Cell::new("Registrar"),
        Cell::new("Host Registration Date"),
        Cell::new("Registrar Abuse Address"),
        Cell::new("IP"),
        Cell::new("IP Provider"),
        Cell::new("Provider Abuse Address"),
        Cell::new(""),
    ]));

    let mut nodes_as_rows: Vec<(usize, Vec<Cell>)> = data
        .parsed_mail
        .delivery_nodes
        .iter()
        .map(|node| {
            let reg_name_cell = if let Some(observed_sender) = &node.observed_sender {
                registrar_name_cell(observed_sender.registrar.as_ref())
            } else {
                Cell::new("N/A")
            };

            let reg_date_cell = if let Some(observed_sender) = &node.observed_sender {
                registration_date_cell(observed_sender.domain.as_ref())
            } else {
                Cell::new("N/A")
            };

            let reg_abuse_cell = if let Some(observed_sender) = &node.observed_sender {
                registrar_abuse_email_cell(observed_sender.registrar.as_ref())
            } else {
                Cell::new("N/A")
            };

            let ip_provider_cell = if let Some(observed_sender) = &node.observed_sender {
                ip_provider_name_cell(observed_sender.infrastructure_provider.as_ref())
            } else {
                Cell::new("N/A")
            };

            let ip_abuse_cell = if let Some(observed_sender) = &node.observed_sender {
                ip_provider_abuse_email_cell(observed_sender.infrastructure_provider.as_ref())
            } else {
                Cell::new("N/A")
            };

            let trusted_cell = Cell::new(if node.trusted { "Yes" } else { "No" });

            (
                node.position,
                vec![
                    Cell::new(&display_recipient(node.recipient.as_ref())),
                    Cell::new(&display_host(node.advertised_sender.as_ref())),
                    Cell::new(&display_host(node.observed_sender.as_ref())),
                    reg_name_cell,
                    reg_date_cell,
                    reg_abuse_cell,
                    Cell::new(&display_ip(node.observed_sender.as_ref())),
                    ip_provider_cell,
                    ip_abuse_cell,
                    trusted_cell,
                ],
            )
        })
        .collect();

    nodes_as_rows.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, row) in nodes_as_rows {
        table.add_row(Row::new(row));
    }

    table_to_string(&table)
}

#[cfg(test)]
mod display_authentication_results_tests {
    use super::*;
    use crate::authentication_results::{AuthenticationResults, Dkim, DkimResult, Spf, SpfResult};
    use crate::data::{EmailAddresses, ParsedMail};
    use crate::message_source::MessageSource;

    #[test]
    fn displays_authentication_results_with_no_authentication_results() {
        let data = build_output_data(None);

        assert_eq!(
            String::from(
                "\
            +--------------------+------------+-----------+-----------------+\n\
            | Service Identifier | N/A                                      |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | DKIM                                                          |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | Result             | Selector   | Signature | User Identifier |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | N/A                | N/A        | N/A       | N/A             |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | SPF                                                           |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | Result             | IP Address | Mail From                   |\n\
            +--------------------+------------+-----------+-----------------+\n\
            | N/A                | N/A        | N/A                         |\n\
            +--------------------+------------+-----------+-----------------+\n\
            "
            ),
            display_authentication_results(&data).unwrap()
        );
    }

    #[test]
    fn displays_authentications_results_with_full_authentication_results() {
        let data = build_output_data(authentication_results());

        assert_eq!(
            String::from(
                "\
            +--------------------+---------------+----------------+-----------------+\n\
            | Service Identifier | mx.google.com                                    |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | DKIM                                                                  |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | Result             | Selector      | Signature      | User Identifier |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | Pass               | dkim_selector | dkim_signature | dkim_user       |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | SPF                                                                   |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | Result             | IP Address    | Mail From                        |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            | SoftFail           | 10.10.10.10   | mailfrom                         |\n\
            +--------------------+---------------+----------------+-----------------+\n\
            "
            ),
            display_authentication_results(&data).unwrap()
        );
    }

    fn build_output_data(authentication_results: Option<AuthenticationResults>) -> OutputData {
        OutputData {
            parsed_mail: ParsedMail {
                authentication_results,
                delivery_nodes: vec![],
                fulfillment_nodes: vec![],
                subject: None,
                email_addresses: EmailAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                    links: vec![],
                },
            },
            message_source: MessageSource::new(""),
            reportable_entities: None,
            run_id: None,
        }
    }

    fn authentication_results() -> Option<AuthenticationResults> {
        Some(AuthenticationResults {
            dkim: Some(Dkim {
                result: Some(DkimResult::Pass),
                selector: Some("dkim_selector".into()),
                signature_snippet: Some("dkim_signature".into()),
                user_identifier_snippet: Some("dkim_user".into()),
            }),
            service_identifier: Some("mx.google.com".into()),
            spf: Some(Spf {
                ip_address: Some("10.10.10.10".into()),
                result: Some(SpfResult::SoftFail),
                smtp_mailfrom: Some("mailfrom".into()),
            }),
        })
    }
}

pub fn display_authentication_results(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();
    let auth_results = data.parsed_mail.authentication_results.as_ref();

    table.add_row(Row::new(vec![
        Cell::new("Service Identifier"),
        authentication_results_service_identifier(auth_results).with_hspan(3),
    ]));

    table.add_row(Row::new(vec![Cell::new("DKIM").with_hspan(4)]));
    table.add_row(Row::new(vec![
        Cell::new("Result"),
        Cell::new("Selector"),
        Cell::new("Signature"),
        Cell::new("User Identifier"),
    ]));

    table.add_row(Row::new(vec![
        authentication_results_dkim_result(auth_results),
        authentication_results_dkim_selector(auth_results),
        authentication_results_dkim_signature(auth_results),
        authentication_results_dkim_user(auth_results),
    ]));
    // table.add_row(Row::new(vec![Cell::new("N/A"), Cell::new("N/A"), Cell::new("N/A"), Cell::new("N/A")]));
    table.add_row(Row::new(vec![Cell::new("SPF").with_hspan(4)]));
    table.add_row(Row::new(vec![
        Cell::new("Result"),
        Cell::new("IP Address"),
        Cell::new("Mail From").with_hspan(2),
    ]));

    table.add_row(Row::new(vec![
        authentication_results_spf_result(auth_results),
        authentication_results_spf_ip_address(auth_results),
        authentication_results_spf_mailfrom(auth_results).with_hspan(2),
    ]));

    table_to_string(&table)
}

#[cfg(test)]
mod display_host_tests {
    use super::*;
    use crate::data::HostNode;

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
    if let Some(HostNode {
        host: Some(host_val),
        ..
    }) = node_option
    {
        String::from(host_val)
    } else {
        String::from("N/A")
    }
}

#[cfg(test)]
mod display_ip_tests {
    use super::*;
    use crate::data::HostNode;

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
    if let Some(HostNode {
        ip_address: Some(ip_val),
        ..
    }) = node_option
    {
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

#[cfg(test)]
mod authentication_results_service_identifier_tests {
    use super::*;

    #[test]
    fn returns_na_if_no_authentication_results() {
        assert_eq!(
            Cell::new("N/A"),
            authentication_results_service_identifier(None)
        );
    }

    #[test]
    fn returns_na_if_no_service_identifier() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_service_identifier(Some(&results))
        );
    }

    #[test]
    fn returns_cell_containing_service_identifier() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: Some(String::from("mx.google.com")),
            spf: None,
        };

        assert_eq!(
            Cell::new("mx.google.com"),
            authentication_results_service_identifier(Some(&results))
        );
    }
}

fn authentication_results_service_identifier(
    results_option: Option<&AuthenticationResults>,
) -> Cell {
    match results_option {
        Some(AuthenticationResults {
            service_identifier, ..
        }) => optional_cell(service_identifier.as_deref()),
        None => Cell::new("N/A"),
    }
}

#[cfg(test)]
mod authentication_results_dkim_result_tests {
    use super::*;
    use crate::authentication_results::{Dkim, DkimResult};

    #[test]
    fn returns_na_if_no_authentication_results() {
        assert_eq!(Cell::new("N/A"), authentication_results_dkim_result(None));
    }

    #[test]
    fn returns_na_if_no_dkim() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_dkim_result(Some(&results))
        );
    }

    #[test]
    fn returns_result() {
        let dkim = Dkim {
            result: Some(DkimResult::TempError),
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: None,
        };

        let results = AuthenticationResults {
            dkim: Some(dkim),
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("TempError"),
            authentication_results_dkim_result(Some(&results))
        );
    }
}

fn authentication_results_dkim_result(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.dkim.as_ref() {
            Some(dkim) => optional_cell(dkim.result.as_ref().map(|r| r.to_string()).as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_dkim_selector_tests {
    use super::*;
    use crate::authentication_results::Dkim;

    #[test]
    fn returns_na_if_no_results() {
        assert_eq!(Cell::new("N/A"), authentication_results_dkim_selector(None));
    }

    #[test]
    fn returns_na_if_no_dkim() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_dkim_selector(Some(&results))
        );
    }

    #[test]
    fn returns_selector() {
        let dkim = Dkim {
            result: None,
            selector: Some("foo".into()),
            signature_snippet: None,
            user_identifier_snippet: None,
        };

        let results = AuthenticationResults {
            dkim: Some(dkim),
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("foo"),
            authentication_results_dkim_selector(Some(&results))
        );
    }
}

fn authentication_results_dkim_selector(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.dkim.as_ref() {
            Some(dkim) => optional_cell(dkim.selector.as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_dkim_signature_tests {
    use super::*;
    use crate::authentication_results::Dkim;

    #[test]
    fn returns_na_if_no_results() {
        assert_eq!(
            Cell::new("N/A"),
            authentication_results_dkim_signature(None)
        );
    }

    #[test]
    fn returns_na_if_no_dkim() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_dkim_signature(Some(&results))
        );
    }

    #[test]
    fn returns_selector() {
        let dkim = Dkim {
            result: None,
            selector: None,
            signature_snippet: Some("foo".into()),
            user_identifier_snippet: None,
        };

        let results = AuthenticationResults {
            dkim: Some(dkim),
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("foo"),
            authentication_results_dkim_signature(Some(&results))
        );
    }
}

fn authentication_results_dkim_signature(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.dkim.as_ref() {
            Some(dkim) => optional_cell(dkim.signature_snippet.as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_dkim_user_tests {
    use super::*;
    use crate::authentication_results::Dkim;

    #[test]
    fn returns_na_if_no_results() {
        assert_eq!(Cell::new("N/A"), authentication_results_dkim_user(None));
    }

    #[test]
    fn returns_na_if_no_dkim() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_dkim_user(Some(&results))
        );
    }

    #[test]
    fn returns_selector() {
        let dkim = Dkim {
            result: None,
            selector: None,
            signature_snippet: None,
            user_identifier_snippet: Some("foo".into()),
        };

        let results = AuthenticationResults {
            dkim: Some(dkim),
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("foo"),
            authentication_results_dkim_user(Some(&results))
        );
    }
}

fn authentication_results_dkim_user(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.dkim.as_ref() {
            Some(dkim) => optional_cell(dkim.user_identifier_snippet.as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_spf_result_tests {
    use super::*;
    use crate::authentication_results::{Spf, SpfResult};

    #[test]
    fn returns_na_if_no_authentication_results() {
        assert_eq!(Cell::new("N/A"), authentication_results_spf_result(None));
    }

    #[test]
    fn returns_na_if_no_spf() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_spf_result(Some(&results))
        );
    }

    #[test]
    fn returns_result() {
        let spf = Spf {
            ip_address: None,
            result: Some(SpfResult::HardFail),
            smtp_mailfrom: None,
        };

        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: Some(spf),
        };

        assert_eq!(
            Cell::new("HardFail"),
            authentication_results_spf_result(Some(&results))
        );
    }
}

fn authentication_results_spf_result(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.spf.as_ref() {
            Some(spf) => optional_cell(spf.result.as_ref().map(|r| r.to_string()).as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_spf_ip_address_tests {
    use super::*;
    use crate::authentication_results::Spf;

    #[test]
    fn returns_na_if_no_results() {
        assert_eq!(
            Cell::new("N/A"),
            authentication_results_spf_ip_address(None)
        );
    }

    #[test]
    fn returns_na_if_no_spf() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_spf_ip_address(Some(&results))
        );
    }

    #[test]
    fn returns_ip_address() {
        let spf = Spf {
            ip_address: Some("10.10.10.10".into()),
            result: None,
            smtp_mailfrom: None,
        };

        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: Some(spf),
        };

        assert_eq!(
            Cell::new("10.10.10.10"),
            authentication_results_spf_ip_address(Some(&results))
        );
    }
}

fn authentication_results_spf_ip_address(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.spf.as_ref() {
            Some(spf) => optional_cell(spf.ip_address.as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

#[cfg(test)]
mod authentication_results_spf_mailfrom_tests {
    use super::*;
    use crate::authentication_results::Spf;

    #[test]
    fn returns_na_if_no_authentication_results() {
        assert_eq!(Cell::new("N/A"), authentication_results_spf_mailfrom(None));
    }

    #[test]
    fn returns_na_if_no_spf() {
        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: None,
        };

        assert_eq!(
            Cell::new("N/A"),
            authentication_results_spf_mailfrom(Some(&results))
        );
    }

    #[test]
    fn returns_mailfrom() {
        let spf = Spf {
            ip_address: None,
            result: None,
            smtp_mailfrom: Some("foo".into()),
        };

        let results = AuthenticationResults {
            dkim: None,
            service_identifier: None,
            spf: Some(spf),
        };

        assert_eq!(
            Cell::new("foo"),
            authentication_results_spf_mailfrom(Some(&results))
        );
    }
}

fn authentication_results_spf_mailfrom(results_option: Option<&AuthenticationResults>) -> Cell {
    match results_option {
        Some(results) => match results.spf.as_ref() {
            Some(spf) => optional_cell(spf.smtp_mailfrom.as_deref()),
            None => optional_cell(None),
        },
        None => optional_cell(None),
    }
}

fn optional_cell(value_option: Option<&str>) -> Cell {
    match value_option {
        Some(value) => Cell::new(value),
        None => Cell::new("N/A"),
    }
}

#[cfg(test)]
mod optional_cell_tests {
    use super::*;

    #[test]
    fn returns_na_if_none() {
        assert_eq!(Cell::new("N/A"), optional_cell(None));
    }

    #[test]
    fn returns_value_if_some() {
        assert_eq!(Cell::new("foo"), optional_cell(Some("foo")));
    }
}

pub fn display_run(run: &Run) -> AppResult<String> {
    Ok(
        format!(
            "{}\n{}",
            run_details(run).unwrap(),
            display_reportable_entities(run).unwrap()
            // email_addresses_details(run).unwrap(),
            // delivery_nodes_details(run).unwrap(),
            // fulfillment_nodes_details(run).unwrap()
        )
    )
}

#[cfg(test)]
mod display_run_tests {
    use chrono::prelude::*;
    use crate::data::{DeliveryNode, DomainCategory, EmailAddresses, ParsedMail, ReportableEntities};
    use crate::message_source::MessageSource;
    use crate::run::Run;
    use super::*;

    #[test]
    fn provides_a_human_friendly_representation_of_run() {
        let run = build_run();

        assert_eq!(
            String::from("\
            +------------+-------------------------+\n\
            | Run ID     | 1234                    |\n\
            +------------+-------------------------+\n\
            | Created At | 2023-08-29 09:41:30 UTC |\n\
            +------------+-------------------------+\n\
            \n\
            +-------------+-------------------+\n\
            | Email Addresses                 |\n\
            +-------------+-------------------+\n\
            | From        | from.1@test.com   |\n\
            |             | from.2@test.com   |\n\
            +-------------+-------------------+\n\
            | Links       | link.1@test.com   |\n\
            |             | link.2@test.com   |\n\
            +-------------+-------------------+\n\
            | Reply-To    | reply.1@test.com  |\n\
            |             | reply.2@test.com  |\n\
            +-------------+-------------------+\n\
            | Return-Path | return.1@test.com |\n\
            |             | return.2@test.com |\n\
            +-------------+-------------------+\n\
            \n\
            +-------------------------+---------------------+-------------------------+\n\
            | Delivery Nodes                                                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Position                | 1                                             |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Recipient               | recipient.1.test.com                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Time                    | 2023-08-29 09:41:01 UTC                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Trusted                 | false                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Advertised Sender                                                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 1.advertised.host.com                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 10.10.10.1                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.advertised.1.com      |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider advertised 1   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar advertised 1  |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Observed Sender                                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 1.observed.host.com                           |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 20.20.20.1                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.observed.1.com        |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider observed 1     |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar observed 1    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Position                | 2                                             |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Recipient               | recipient.2.test.com                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Time                    | 2023-08-29 09:41:02 UTC                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Trusted                 | false                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Advertised Sender                                                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 2.advertised.host.com                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 10.10.10.2                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.advertised.2.com      |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider advertised 2   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar advertised 2  |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Observed Sender                                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 2.observed.host.com                           |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 20.20.20.2                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.observed.2.com        |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider observed 2     |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar observed 2    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            \n\
            +-----------+---------------------+-------------------------+\n\
            | Fulfillment Nodes                                         |\n\
            +-----------+---------------------+-------------------------+\n\
            | Hidden    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.hidden.1@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.hidden.1.com          |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.hidden.1@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar hidden 1      |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://hidden-1.test.com                     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Visible   |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.visible.1@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.visible.1.com         |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.visible.1@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar visible 1     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://visible-1.test.com                    |\n\
            +-----------+---------------------+-------------------------+\n\
            | Hidden    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.hidden.2@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.hidden.2.com          |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.hidden.2@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar hidden 2      |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://hidden-2.test.com                     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Visible   |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.visible.2@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.visible.2.com         |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.visible.2@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar visible 2     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://visible-2.test.com                    |\n\
            +-----------+---------------------+-------------------------+\n\
            "),
            display_run(&run).unwrap()
        )
    }

    fn build_run() -> Run {
        let reportable_entities = ReportableEntities {
            delivery_nodes: vec![
                build_delivery_node(1),
                build_delivery_node(2),
            ],
            email_addresses: EmailAddresses {
                from: vec![
                    EmailAddresses::to_email_address_data("from.1@test.com".into()),
                    EmailAddresses::to_email_address_data("from.2@test.com".into()),
                ],
                links: vec![
                    EmailAddresses::to_email_address_data("link.1@test.com".into()),
                    EmailAddresses::to_email_address_data("link.2@test.com".into()),
                ],
                reply_to: vec![
                    EmailAddresses::to_email_address_data("reply.1@test.com".into()),
                    EmailAddresses::to_email_address_data("reply.2@test.com".into()),
                ],
                return_path: vec![
                    EmailAddresses::to_email_address_data("return.1@test.com".into()),
                    EmailAddresses::to_email_address_data("return.2@test.com".into()),
                ],
            },
            fulfillment_nodes: vec![
                build_fulfillment_node(1),
                build_fulfillment_node(2),
            ],
        };

        Run {
            id: 1234,
            created_at: Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 30).unwrap(),
            data: OutputData {
                message_source: MessageSource::new(""),
                parsed_mail: ParsedMail {
                    authentication_results: None,
                    delivery_nodes: vec![],
                    email_addresses: EmailAddresses {
                        from: vec![],
                        links: vec![],
                        reply_to: vec![],
                        return_path: vec![],
                    },
                    fulfillment_nodes: vec![],
                    subject: None,
                },
                reportable_entities: Some(reportable_entities),
                run_id: None,
            },
            message_source: MessageSource::new("")
        }
    }

    fn build_delivery_node(position: usize) -> DeliveryNode {
        let time = Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, position.try_into().unwrap()).unwrap();

        DeliveryNode {
            advertised_sender: Some(build_host_node("advertised", position)),
            observed_sender: Some(build_host_node("observed", position)),
            position,
            recipient: Some(format!("recipient.{}.test.com", position)),
            time: Some(time),
            trusted: false
        }
    }

    fn build_host_node(sender_type: &str, position: usize) -> HostNode {
        HostNode {
            domain: Some(build_domain(sender_type, position)),
            host: Some(build_host(sender_type, position)),
            infrastructure_provider: Some(build_infrastructure_provider(sender_type, position)),
            ip_address: Some(build_ip_address(sender_type, position)),
            registrar: Some(build_registrar(sender_type, position)),
        }
    }

    fn build_host(sender_type: &str, position: usize) -> String {
        format!("{position}.{sender_type}.host.com")
    }

    fn build_domain(sender_type: &str, position: usize) -> Domain {
        let registration_date = Utc
            .with_ymd_and_hms(2023, 6, 1, 10, 10, position.try_into().unwrap())
            .unwrap();

        Domain {
            abuse_email_address: Some(format!("d.{sender_type}.{position}@test.com")),
            category: DomainCategory::Other,
            name: format!("d.{sender_type}.{position}.com"),
            registration_date: Some(registration_date),
        }
    }

    fn build_infrastructure_provider(sender_type: &str, position: usize) -> InfrastructureProvider {
        InfrastructureProvider {
            abuse_email_address: Some(format!("i.{sender_type}.{position}@test.com")),
            name: Some(format!("Provider {sender_type} {position}")),
        }
    }

    fn build_ip_address(sender_type: &str, position: usize) -> String {
        if sender_type == "advertised" {
            format!("10.10.10.{position}")
        } else {
            format!("20.20.20.{position}")
        }
    }

    fn build_registrar(sender_type: &str, position: usize) -> Registrar {
        Registrar {
            abuse_email_address: Some(format!("r.{sender_type}.{position}@test.com")),
            name: Some(format!("Registrar {sender_type} {position}")),
        }
    }

    fn build_fulfillment_node(position: usize) -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(build_node("hidden", position)),
            visible: build_node("visible", position),
        }
    }

    fn build_node(label: &str, position: usize) -> Node {
        Node {
            domain: Some(build_domain(label, position)),
            registrar: Some(build_registrar(label, position)),
            url: format!("https://{label}-{position}.test.com")
        }
    }
}

fn run_details(run: &Run) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![
            Cell::new("Run ID"),
            Cell::new(&run.id.to_string()),
        ])
    );
    table.add_row(
        Row::new(vec![
            Cell::new("Created At"),
            Cell::new(&run.created_at.to_string()),
        ])
    );

    table_to_string(&table)
}

pub fn display_reportable_entities(run: &Run) -> AppResult<String> {
    Ok(
        format!(
            "{}\n{}\n{}",
            email_addresses_details(run).unwrap(),
            delivery_nodes_details(run).unwrap(),
            fulfillment_nodes_details(run).unwrap()
        )
    )
}

#[cfg(test)]
mod display_reportable_entities_tests {
    use chrono::prelude::*;
    use crate::data::{DeliveryNode, DomainCategory, EmailAddresses, ParsedMail, ReportableEntities};
    use crate::message_source::MessageSource;
    use crate::run::Run;
    use super::*;

    #[test]
    fn provides_a_human_friendly_representation_of_reportable_entities() {
        let run = build_run();

        assert_eq!(
            String::from("\
            +-------------+-------------------+\n\
            | Email Addresses                 |\n\
            +-------------+-------------------+\n\
            | From        | from.1@test.com   |\n\
            |             | from.2@test.com   |\n\
            +-------------+-------------------+\n\
            | Links       | link.1@test.com   |\n\
            |             | link.2@test.com   |\n\
            +-------------+-------------------+\n\
            | Reply-To    | reply.1@test.com  |\n\
            |             | reply.2@test.com  |\n\
            +-------------+-------------------+\n\
            | Return-Path | return.1@test.com |\n\
            |             | return.2@test.com |\n\
            +-------------+-------------------+\n\
            \n\
            +-------------------------+---------------------+-------------------------+\n\
            | Delivery Nodes                                                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Position                | 1                                             |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Recipient               | recipient.1.test.com                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Time                    | 2023-08-29 09:41:01 UTC                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Trusted                 | false                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Advertised Sender                                                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 1.advertised.host.com                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 10.10.10.1                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.advertised.1.com      |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider advertised 1   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.advertised.1@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar advertised 1  |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Observed Sender                                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 1.observed.host.com                           |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 20.20.20.1                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.observed.1.com        |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider observed 1     |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.observed.1@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar observed 1    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Position                | 2                                             |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Recipient               | recipient.2.test.com                          |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Time                    | 2023-08-29 09:41:02 UTC                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Trusted                 | false                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Advertised Sender                                                       |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 2.advertised.host.com                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 10.10.10.2                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.advertised.2.com      |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider advertised 2   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.advertised.2@test.com |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar advertised 2  |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Observed Sender                                                         |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Host                    | 2.observed.host.com                           |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | IP Address              | 20.20.20.2                                    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Domain                  |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | d.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Category            | Other                   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | d.observed.2.com        |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Infrastructure Provider |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | i.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Provider observed 2     |\n\
            +-------------------------+---------------------+-------------------------+\n\
            | Registrar               |                                               |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Abuse Email Address | r.observed.2@test.com   |\n\
            +-------------------------+---------------------+-------------------------+\n\
            |                         | Name                | Registrar observed 2    |\n\
            +-------------------------+---------------------+-------------------------+\n\
            \n\
            +-----------+---------------------+-------------------------+\n\
            | Fulfillment Nodes                                         |\n\
            +-----------+---------------------+-------------------------+\n\
            | Hidden    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.hidden.1@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.hidden.1.com          |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.hidden.1@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar hidden 1      |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://hidden-1.test.com                     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Visible   |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.visible.1@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.visible.1.com         |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:01 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.visible.1@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar visible 1     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://visible-1.test.com                    |\n\
            +-----------+---------------------+-------------------------+\n\
            | Hidden    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.hidden.2@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.hidden.2.com          |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.hidden.2@test.com     |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar hidden 2      |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://hidden-2.test.com                     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Visible   |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            | Domain    |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | d.visible.2@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Category            | Other                   |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | d.visible.2.com         |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Registration Date   | 2023-06-01 10:10:02 UTC |\n\
            +-----------+---------------------+-------------------------+\n\
            | Registrar |                                               |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Abuse Email Address | r.visible.2@test.com    |\n\
            +-----------+---------------------+-------------------------+\n\
            |           | Name                | Registrar visible 2     |\n\
            +-----------+---------------------+-------------------------+\n\
            | Url       | https://visible-2.test.com                    |\n\
            +-----------+---------------------+-------------------------+\n\
            "),
            display_reportable_entities(&run).unwrap()
        )
    }

    fn build_run() -> Run {
        let reportable_entities = ReportableEntities {
            delivery_nodes: vec![
                build_delivery_node(1),
                build_delivery_node(2),
            ],
            email_addresses: EmailAddresses {
                from: vec![
                    EmailAddresses::to_email_address_data("from.1@test.com".into()),
                    EmailAddresses::to_email_address_data("from.2@test.com".into()),
                ],
                links: vec![
                    EmailAddresses::to_email_address_data("link.1@test.com".into()),
                    EmailAddresses::to_email_address_data("link.2@test.com".into()),
                ],
                reply_to: vec![
                    EmailAddresses::to_email_address_data("reply.1@test.com".into()),
                    EmailAddresses::to_email_address_data("reply.2@test.com".into()),
                ],
                return_path: vec![
                    EmailAddresses::to_email_address_data("return.1@test.com".into()),
                    EmailAddresses::to_email_address_data("return.2@test.com".into()),
                ],
            },
            fulfillment_nodes: vec![
                build_fulfillment_node(1),
                build_fulfillment_node(2),
            ],
        };

        Run {
            id: 1234,
            created_at: Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 30).unwrap(),
            data: OutputData {
                message_source: MessageSource::new(""),
                parsed_mail: ParsedMail {
                    authentication_results: None,
                    delivery_nodes: vec![],
                    email_addresses: EmailAddresses {
                        from: vec![],
                        links: vec![],
                        reply_to: vec![],
                        return_path: vec![],
                    },
                    fulfillment_nodes: vec![],
                    subject: None,
                },
                reportable_entities: Some(reportable_entities),
                run_id: None,
            },
            message_source: MessageSource::new("")
        }
    }

    fn build_delivery_node(position: usize) -> DeliveryNode {
        let time = Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, position.try_into().unwrap()).unwrap();

        DeliveryNode {
            advertised_sender: Some(build_host_node("advertised", position)),
            observed_sender: Some(build_host_node("observed", position)),
            position,
            recipient: Some(format!("recipient.{}.test.com", position)),
            time: Some(time),
            trusted: false
        }
    }

    fn build_host_node(sender_type: &str, position: usize) -> HostNode {
        HostNode {
            domain: Some(build_domain(sender_type, position)),
            host: Some(build_host(sender_type, position)),
            infrastructure_provider: Some(build_infrastructure_provider(sender_type, position)),
            ip_address: Some(build_ip_address(sender_type, position)),
            registrar: Some(build_registrar(sender_type, position)),
        }
    }

    fn build_host(sender_type: &str, position: usize) -> String {
        format!("{position}.{sender_type}.host.com")
    }

    fn build_domain(sender_type: &str, position: usize) -> Domain {
        let registration_date = Utc
            .with_ymd_and_hms(2023, 6, 1, 10, 10, position.try_into().unwrap())
            .unwrap();

        Domain {
            abuse_email_address: Some(format!("d.{sender_type}.{position}@test.com")),
            category: DomainCategory::Other,
            name: format!("d.{sender_type}.{position}.com"),
            registration_date: Some(registration_date),
        }
    }

    fn build_infrastructure_provider(sender_type: &str, position: usize) -> InfrastructureProvider {
        InfrastructureProvider {
            abuse_email_address: Some(format!("i.{sender_type}.{position}@test.com")),
            name: Some(format!("Provider {sender_type} {position}")),
        }
    }

    fn build_ip_address(sender_type: &str, position: usize) -> String {
        if sender_type == "advertised" {
            format!("10.10.10.{position}")
        } else {
            format!("20.20.20.{position}")
        }
    }

    fn build_registrar(sender_type: &str, position: usize) -> Registrar {
        Registrar {
            abuse_email_address: Some(format!("r.{sender_type}.{position}@test.com")),
            name: Some(format!("Registrar {sender_type} {position}")),
        }
    }

    fn build_fulfillment_node(position: usize) -> FulfillmentNode {
        FulfillmentNode {
            hidden: Some(build_node("hidden", position)),
            visible: build_node("visible", position),
        }
    }

    fn build_node(label: &str, position: usize) -> Node {
        Node {
            domain: Some(build_domain(label, position)),
            registrar: Some(build_registrar(label, position)),
            url: format!("https://{label}-{position}.test.com")
        }
    }
}

fn email_addresses_details(run: &Run) -> AppResult<String> {
    let mut table = Table::new();

    if let Some(reportable_entities) = &run.data.reportable_entities {
        let email_addresses = &reportable_entities.email_addresses;
        table.add_row(
            Row::new(vec![
                Cell::new("Email Addresses").with_hspan(2),
            ])
        );
        table.add_row(
            build_email_addresses_row(
                "From",
                &email_addresses.from
            )
        );
        table.add_row(
            build_email_addresses_row(
                "Links",
                &email_addresses.links
            )
        );
        table.add_row(
            build_email_addresses_row(
                "Reply-To",
                &email_addresses.reply_to
            )
        );
        table.add_row(
            build_email_addresses_row(
                "Return-Path",
                &email_addresses.return_path
            )
        );
    }

    table_to_string(&table)
}

fn build_email_addresses_row(label: &str, email_addresses: &[EmailAddressData]) -> Row {
    let addresses_string = email_addresses
        .iter()
        .map(|address| address.address.as_str())
        .collect::<Vec<&str>>()
        .join("\n");

    Row::new(vec![
        Cell::new(label),
        Cell::new(&addresses_string),
    ])
}

#[cfg(test)]
mod build_email_addresses_row_tests {
    use super::*;
    use crate::data::EmailAddressData;

    #[test]
    fn converts_email_addresses_into_row_of_cells() {
        let email_addresses = vec![
            EmailAddressData::from_email_address("address_1@test.com"),
            EmailAddressData::from_email_address("address_2@test.com"),
        ];

        let expected = Row::new(vec![
            Cell::new("xx"),
            Cell::new("address_1@test.com\naddress_2@test.com"),
        ]);

        assert_eq!(expected, build_email_addresses_row("xx", &email_addresses));
    }
}

fn delivery_nodes_details(run: &Run) -> AppResult<String> {
    let mut table = Table::new();

    if let Some(reportable_entities) = &run.data.reportable_entities {
        let delivery_nodes = &reportable_entities.delivery_nodes;

        table.add_row(
            Row::new(vec![
                Cell::new("Delivery Nodes").with_hspan(3),
            ])
        );

        for node in delivery_nodes {
            add_delivery_node_rows(&mut table, node);
        }
    }

    table_to_string(&table)
}

fn add_delivery_node_rows(table: &mut Table, node: &DeliveryNode) {
    table.add_row(
        Row::new(vec![
            Cell::new("Position"),
            Cell::new(&node.position.to_string()).with_hspan(2),
        ])
    );
    table.add_row(
        Row::new(vec![
            Cell::new("Recipient"),
            optional_cell(node.recipient.as_deref()).with_hspan(2),
        ])
    );
    table.add_row(
        Row::new(vec![
            Cell::new("Time"),
            optional_date_time_cell(node.time.as_ref()).with_hspan(2),
        ])
    );
    table.add_row(
        Row::new(vec![
            Cell::new("Trusted"),
            Cell::new(&node.trusted.to_string()).with_hspan(2),
        ])
    );

    add_host_node_rows(table, "Advertised Sender", node.advertised_sender.as_ref());
    add_host_node_rows(table, "Observed Sender", node.observed_sender.as_ref());
}

fn add_host_node_rows(table: &mut Table, label: &str, node_option: Option<&HostNode>) {
    if let Some(node) = node_option {
        table.add_row(
            Row::new(vec![
                Cell::new(label).with_hspan(3),
            ])
        );
        table.add_row(
            Row::new(vec![
                Cell::new("Host"),
                optional_cell(node.host.as_deref()).with_hspan(2),
            ])
        );
        table.add_row(
            Row::new(vec![
                Cell::new("IP Address"),
                optional_cell(node.ip_address.as_deref()).with_hspan(2),
            ])
        );
        add_domain_rows(table, node.domain.as_ref());
        add_infrastructure_provider_rows(table, node.infrastructure_provider.as_ref());
        add_registrar_rows(table, node.registrar.as_ref());
    }
}

fn add_domain_rows(table: &mut Table, domain_option: Option<&Domain>) {
    if let Some(domain) = domain_option {
        table.add_row(
            Row::new(vec![
                Cell::new("Domain"),
                Cell::new("").with_hspan(2),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Abuse Email Address"),
                optional_cell(domain.abuse_email_address.as_deref()),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Category"),
                Cell::new(&domain.category.to_string()),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Name"),
                Cell::new(&domain.name),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Registration Date"),
                optional_date_time_cell(domain.registration_date.as_ref()),
            ])
        );
    }
}

fn add_infrastructure_provider_rows(
    table: &mut Table,
    provider_option: Option<&InfrastructureProvider>
) {
    if let Some(provider) = provider_option {
        table.add_row(
            Row::new(vec![
                Cell::new("Infrastructure Provider"),
                Cell::new("").with_hspan(2),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Abuse Email Address"),
                optional_cell(provider.abuse_email_address.as_deref()),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Name"),
                optional_cell(provider.name.as_deref()),
            ])
        );
    }
}

fn add_registrar_rows(table: &mut Table, registrar_option: Option<&Registrar>) {
    if let Some(registrar) = registrar_option {
        table.add_row(
            Row::new(vec![
                Cell::new("Registrar"),
                Cell::new("").with_hspan(2),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Abuse Email Address"),
                optional_cell(registrar.abuse_email_address.as_deref()),
            ])
        );

        table.add_row(
            Row::new(vec![
                Cell::new(""),
                Cell::new("Name"),
                optional_cell(registrar.name.as_deref()),
            ])
        );
    }
}

fn fulfillment_nodes_details(run: &Run) -> AppResult<String> {
    let mut table = Table::new();

    if let Some(reportable_entities) = &run.data.reportable_entities {
        let fulfillment_nodes = &reportable_entities.fulfillment_nodes;

        table.add_row(
            Row::new(vec![
                Cell::new("Fulfillment Nodes").with_hspan(3),
            ])
        );

        for node in fulfillment_nodes {
            add_fulfillment_node_rows(&mut table, node);
        }
    }

    table_to_string(&table)
}

fn add_fulfillment_node_rows(table: &mut Table, node: &FulfillmentNode) {
    if let Some(hidden) = node.hidden.as_ref() {
        table.add_row(
            Row::new(vec![
                Cell::new("Hidden"),
                Cell::new("").with_hspan(2)
            ])
        );

        add_node_rows(table, hidden);
    }

    table.add_row(
        Row::new(vec![
            Cell::new("Visible"),
            Cell::new("").with_hspan(2)
        ])
    );

    add_node_rows(table, &node.visible);
}

fn add_node_rows(table: &mut Table, node: &Node) {
    add_domain_rows(table, node.domain.as_ref());

    add_registrar_rows(table, node.registrar.as_ref());

    table.add_row(
        Row::new(vec![
            Cell::new("Url"),
            Cell::new(&node.url).with_hspan(2),
        ])
    );
}

fn optional_date_time_cell(value_option: Option<&DateTime<Utc>>) -> Cell {
    match value_option {
        Some(value) => Cell::new(&value.to_string()),
        None => Cell::new("N/A"),
    }
}

#[cfg(test)]
mod optional_date_time_cell_tests {
    use super::*;

    #[test]
    fn returns_na_if_none() {
        assert_eq!(Cell::new("N/A"), optional_date_time_cell(None));
    }

    #[test]
    fn returns_date_time_if_some() {
        let date_time = Utc.with_ymd_and_hms(2023, 8, 29, 9, 41, 30).unwrap();

        assert_eq!(
            Cell::new("2023-08-29 09:41:30 UTC"),
            optional_date_time_cell(Some(&date_time))
        );
    }
}
