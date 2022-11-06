use assert_cmd::Command;
use predicates::prelude::*;

mod mountebank;

use mountebank::{
    clear_all_impostors,
    setup_bootstrap_server,
    setup_dns_server,
    DnsServerConfig,
};
use chrono::prelude::*;

#[test]
fn test_fetching_rdap_details() {
    let mut cmd = Command::cargo_bin("pp-rdap").unwrap();

    clear_all_impostors();
    setup_bootstrap_server();

    setup_dns_server(
        vec![
            DnsServerConfig {
                domain_name: "fake.net",
                registrar: "Reg One",
                abuse_email: "abuse@regone.zzz",
                registration_date: Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
            },
            DnsServerConfig {
                domain_name: "possiblynotfake.com",
                registrar: "Reg Two",
                abuse_email: "abuse@regtwo.zzz",
                registration_date: Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()
            },
            DnsServerConfig {
                domain_name: "morethanlikelyfake.net",
                registrar: "Reg Three",
                abuse_email: "abuse@regthree.zzz",
                registration_date: Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()
            },
        ]
    );

    cmd
        .args(&["--human"])
        .write_stdin(json_input())
        .env("RDAP_BOOTSTRAP_HOST", "http://localhost:4545")
        .assert()
        .success()
        .stdout(
            predicates::str::contains(
                "abuse@regone.zzz"
            ).and(
                predicates::str::contains("Reg Two")
            ).and(
                predicates::str::contains("2022-11-18 10:11:14")
            )
        );
}

fn json_input() -> String {
    use serde_json::json;

    json!({
        "parsed_mail": {
            "subject": "We’re sorry that we didn’t touch base with you earlier. f309",
            "sender_addresses": {
                "from": "PIBIeSRqUtiEw1NCg4@fake.net",
                "reply_to": "blah@possiblynotfake.com",
                "return_path": "info@morethanlikelyfake.net"
            }
        }
    }).to_string()
}
