use crate::analyser::SenderAddresses;
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
            from: Some("fr@test.com".into()),
            reply_to: Some("rt@test.com".into()),
            return_path: Some("rp@test.com".into())
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

fn table_to_string(table: &Table) -> AppResult<String> {
    let mut buffer: Vec<u8> = Vec::new();

    table.print(&mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

fn row_with_optional_value(table: &mut Table, label: &str, value: &Option<String>) {
    let val = match value {
        Some(v) => v,
        None => "N/A"
    };

    table.add_row(
        Row::new(vec![Cell::new(label), Cell::new(val)])
    );
}
