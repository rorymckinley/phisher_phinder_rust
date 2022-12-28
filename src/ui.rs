use crate::data::{Domain, EmailAddressData, Link, OutputData, Registrar};
use crate::result::AppResult;

use prettytable::{Cell, Row, Table};

#[cfg(test)]
mod display_sender_addresses_extended_tests {
    use super::*;
    use crate::data::{
        Domain,
        DomainCategory,
        EmailAddressData,
        ParsedMail,
        Registrar,
        SenderAddresses
    };
    use chrono::prelude::*;

    #[test]
    fn displays_extended_data_for_sender_addresses() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                links: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: SenderAddresses {
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
                }
            }
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
            "),
            display_sender_addresses_extended(&data).unwrap()
        )
    }

    #[test]
    fn display_extended_data_no_domain_data() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                links: vec![],
                subject: Some("Send me money now! Please?".into()),
                email_addresses: SenderAddresses {
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
                }
            }
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
                    domain_category_cell(domain),
                    registrar_name_cell(registrar),
                    registration_date_cell(domain),
                    registrar_abuse_email_cell(registrar)
                ]
            )
        );
    }
}

#[cfg(test)]
mod display_links_tests {
    use super::*;
    use crate::data::{Link, ParsedMail, SenderAddresses};

    #[test]
    fn display_link_details() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                links: vec![
                    Link::new("https://foo.bar"),
                    Link::new("https://foo.baz"),
                ],
                subject: None,
                email_addresses: SenderAddresses {
                    from: vec![],
                    reply_to: vec![],
                    return_path: vec![],
                }
            }
        };

        assert_eq!(
            String::from("\
            +-----------------+\n\
            | Url             |\n\
            +-----------------+\n\
            | https://foo.bar |\n\
            +-----------------+\n\
            | https://foo.baz |\n\
            +-----------------+\n\
            "),
            display_links(&data).unwrap()
        )
    }
}

pub fn display_links(data: &OutputData) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![
            Cell::new("Url"),
        ])
    );

    for link in data.parsed_mail.links.iter() {
        link_row(&mut table, link);
    }

    table_to_string(&table)
}

fn link_row(table: &mut Table, link: &Link) {
    table.add_row(Row::new(vec![Cell::new(&link.href)]));
}

#[cfg(test)]
mod domain_category_cell_tests {
    use super::*;
    use crate::data::DomainCategory;

    #[test]
    fn returns_n_a_cell_if_domain_is_none() {
        assert_eq!(Cell::new("N/A"), domain_category_cell(&None))
    }

    #[test]
    fn returns_the_category_if_domain_exists() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: None
        };

        assert_eq!(Cell::new("Other"), domain_category_cell(&Some(domain)));
    }
}

fn domain_category_cell(domain_opt: &Option<Domain>) -> Cell {
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
        assert_eq!(Cell::new("N/A"), registration_date_cell(&None))
    }

    #[test]
    fn returns_date_if_domain_has_registration_date() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: Some(datetime(2022, 12, 25, 10, 11, 12))
        };

        assert_eq!(Cell::new("2022-12-25 10:11:12"), registration_date_cell(&Some(domain)));
    }

    #[test]
    fn returns_n_a_if_no_date() {
        let domain = Domain {
            abuse_email_address: None,
            category: DomainCategory::Other,
            name: "doesnotmatter".into(),
            registration_date: None,
        };

        assert_eq!(Cell::new("N/A"), registration_date_cell(&Some(domain)));
    }

    fn datetime(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> chrono::DateTime<Utc> {
        chrono::Utc.with_ymd_and_hms(y, m, d, h, min, s).single().unwrap()
    }
}

fn registration_date_cell(domain_opt: &Option<Domain>) -> Cell {
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
        assert_eq!(Cell::new("N/A"), registrar_name_cell(&None))
    }

    #[test]
    fn returns_registrar_name_if_registrar() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: Some("Reg One".into()),
        };

        assert_eq!(Cell::new("Reg One"), registrar_name_cell(&Some(registrar)));
    }

    #[test]
    fn returns_n_a_if_no_name() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(Cell::new("N/A"), registrar_name_cell(&Some(registrar)));
    }
}

fn registrar_name_cell(registrar_opt: &Option<Registrar>) -> Cell {
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
        assert_eq!(Cell::new("N/A"), registrar_abuse_email_cell(&None));
    }

    #[test]
    fn returns_email_address() {
        let registrar = Registrar {
            abuse_email_address: Some("abuse@regone.co.za".into()),
            name: None,
        };

        assert_eq!(Cell::new("abuse@regone.co.za"), registrar_abuse_email_cell(&Some(registrar)));
    }

    #[test]
    fn returns_n_a_if_abuse_email_address_none() {
        let registrar = Registrar {
            abuse_email_address: None,
            name: None,
        };

        assert_eq!(Cell::new("N/A"), registrar_abuse_email_cell(&Some(registrar)));
    }
}

fn registrar_abuse_email_cell(registrar_opt: &Option<Registrar>) -> Cell {
    if let Some(Registrar {abuse_email_address: Some(abuse_email_address), ..}) = registrar_opt {
        Cell::new(abuse_email_address)
    } else {
        Cell::new("N/A")
    }
}
