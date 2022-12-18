use crate::data::SenderAddresses;
use crate::data::{Domain, EmailAddressData, OutputData};
use crate::result::AppResult;

use prettytable::{Cell, Row, Table};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_sender_addresses_no_addresses() {
        let addresses = SenderAddresses {
            from: None,
            reply_to: None,
            return_path: None,
        };

        assert_eq!(
            String::from("\
            +----------------+--------+\n\
            | Address Source | Values |\n\
            +----------------+--------+\n\
            | From           | N/A    |\n\
            +----------------+--------+\n\
            | Reply-To       | N/A    |\n\
            +----------------+--------+\n\
            | Return-Path    | N/A    |\n\
            +----------------+--------+\n\
            "),
            display_sender_addresses(&addresses).unwrap()
        );
    }

    #[test]
    fn test_display_sender_addresses() {
        let addresses = SenderAddresses {
            from: Some(convert_email_address("fr@test.com")),
            reply_to: Some(convert_email_address("rt@test.com")),
            return_path: Some(convert_email_address("rp@test.com"))
        };

        assert_eq!(
            String::from("\
            +----------------+-------------+\n\
            | Address Source | Values      |\n\
            +----------------+-------------+\n\
            | From           | fr@test.com |\n\
            +----------------+-------------+\n\
            | Reply-To       | rt@test.com |\n\
            +----------------+-------------+\n\
            | Return-Path    | rp@test.com |\n\
            +----------------+-------------+\n\
            "),
            display_sender_addresses(&addresses).unwrap()
        );
    }

    fn convert_email_address(address: &str) -> EmailAddressData {
        SenderAddresses::to_email_address_data(address.into())
    }
}

pub fn display_sender_addresses(sender_addresses: &SenderAddresses) -> AppResult<String> {
    let mut table = Table::new();

    table.add_row(
        Row::new(vec![Cell::new("Address Source"), Cell::new("Values")])
    );

    row_with_optional_value(&mut table, "From", &sender_addresses.from);
    row_with_optional_value(&mut table, "Reply-To", &sender_addresses.reply_to);
    row_with_optional_value(&mut table, "Return-Path", &sender_addresses.return_path);

    table_to_string(&table)
}

#[cfg(test)]
mod display_sender_addresses_extended_tests {
    use super::*;
    use crate::data::{Domain, DomainCategory, EmailAddressData, ParsedMail, SenderAddresses};
    use chrono::prelude::*;

    #[test]
    fn displays_extended_data_for_sender_addresses() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Send me money now! Please?".into()),
                sender_addresses: SenderAddresses {
                    from: Some(
                        EmailAddressData {
                            address: "fr@test.xxx".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.xxx".into(),
                                    registrar: Some("Reg One".into()),
                                    registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                                    abuse_email_address: Some("abuse@regone.zzz".into())
                                }
                            )
                        }
                    ),
                    reply_to: Some(
                        EmailAddressData {
                            address: "rt@test.yyy".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.yyy".into(),
                                    registrar: Some("Reg Two".into()),
                                    registration_date: Some(datetime(2022, 12, 2, 3, 4, 5)),
                                    abuse_email_address: Some("abuse@regtwo.zzz".into())
                                }
                            )
                        }
                    ),
                    return_path: Some(
                        EmailAddressData {
                            address: "rp@test.zzz".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.zzz".into(),
                                    registrar: Some("Reg Three".into()),
                                    registration_date: Some(datetime(2022, 12, 3, 4, 5, 6)),
                                    abuse_email_address: Some("abuse@regthree.zzz".into())
                                }
                            )
                        }
                    ),
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
            | Reply-To       | rt@test.yyy | Other    | Reg Two   | 2022-12-02 03:04:05 | abuse@regtwo.zzz    |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            | Return-Path    | rp@test.zzz | Other    | Reg Three | 2022-12-03 04:05:06 | abuse@regthree.zzz  |\n\
            +----------------+-------------+----------+-----------+---------------------+---------------------+\n\
            "),
            display_sender_addresses_extended(&data).unwrap()
        )
    }

    #[test]
    fn display_extended_data_no_domain_data() {
        let data = OutputData {
            parsed_mail: ParsedMail {
                subject: Some("Send me money now! Please?".into()),
                sender_addresses: SenderAddresses {
                    from: Some(
                        EmailAddressData {
                            address: "fr@test.xxx".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.xxx".into(),
                                    registrar: Some("Reg One".into()),
                                    registration_date: Some(datetime(2022, 12, 1, 2, 3, 4)),
                                    abuse_email_address: Some("abuse@regone.zzz".into())
                                }
                            )
                        }
                    ),
                    reply_to: Some(
                        EmailAddressData {
                            address: "rt@test.yyy".into(),
                            domain: None
                        }
                    ),
                    return_path: Some(
                        EmailAddressData {
                            address: "rp@test.zzz".into(),
                            domain: Some(
                                Domain {
                                    category: DomainCategory::Other,
                                    name: "test.zzz".into(),
                                    registrar: Some("Reg Three".into()),
                                    registration_date: Some(datetime(2022, 12, 3, 4, 5, 6)),
                                    abuse_email_address: Some("abuse@regthree.zzz".into())
                                }
                            )
                        }
                    ),
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

    let addresses = &data.parsed_mail.sender_addresses;

    row_with_optional_values(&mut table, "From", &addresses.from);
    row_with_optional_values(&mut table, "Reply-To", &addresses.reply_to);
    row_with_optional_values(&mut table, "Return-Path", &addresses.return_path);

    table_to_string(&table)
}

fn table_to_string(table: &Table) -> AppResult<String> {
    let mut buffer: Vec<u8> = Vec::new();

    table.print(&mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

fn row_with_optional_value(table: &mut Table, label: &str, value: &Option<EmailAddressData>) {
    let val = match value {
        Some(data) => &data.address,
        None => "N/A"
    };

    table.add_row(
        Row::new(vec![Cell::new(label), Cell::new(val)])
    );
}

fn row_with_optional_values(table: &mut Table, label: &str, email_address_data: &Option<EmailAddressData>) {
    if let Some(EmailAddressData {address, domain: possible_domain}) = email_address_data {
        if let Some(
            Domain {name: _, category, registrar, registration_date, abuse_email_address}
        ) = possible_domain {
            table.add_row(
                Row::new(
                    vec![
                        Cell::new(label),
                        Cell::new(address),
                        Cell::new(&category.to_string()),
                        Cell::new(registrar.as_ref().unwrap()),
                        Cell::new(&registration_date.as_ref().unwrap().format("%Y-%m-%d %H:%M:%S").to_string()),
                        Cell::new(abuse_email_address.as_ref().unwrap())
                    ]
                )
            );
        } else {
            table.add_row(
                Row::new(
                    vec![
                        Cell::new(label),
                        Cell::new(address),
                        Cell::new("N/A"),
                        Cell::new("N/A"),
                        Cell::new("N/A"),
                        Cell::new("N/A"),
                    ]
                )
            );
        }
    }
}
