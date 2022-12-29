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
fn test_fetching_rdap_details() { setup_mountebank();
    setup_mountebank();

    let mut cmd = Command::cargo_bin("pp-rdap").unwrap();

    cmd
        .args(["--human"])
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

#[test]
fn test_fetching_rdap_details_json() {
    setup_mountebank();

    let mut cmd = Command::cargo_bin("pp-rdap").unwrap();

    cmd
        .write_stdin(json_input())
        .env("RDAP_BOOTSTRAP_HOST", "http://localhost:4545")
        .assert()
        .success()
        .stdout(json_output());
}

fn json_input() -> String {
    use serde_json::json;

    json!({
        "parsed_mail": {
            "links": [],
            "subject": "We’re sorry that we didn’t touch base with you earlier. f309",
            "email_addresses": {
                "from": [{
                    "address": "PIBIeSRqUtiEw1NCg4@fake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "fake.net",
                        "registration_date": null,
                    },
                    "registrar": null,
                }],
                "links": [],
                "reply_to": [{
                    "address": "blah@possiblynotfake.com",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "possiblynotfake.com",
                        "registration_date": null,
                    },
                    "registrar": null,
                }],
                "return_path": [{
                    "address": "info@morethanlikelyfake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "morethanlikelyfake.net",
                        "registration_date": null,
                    },
                    "registrar": null,
                }]
            }
        }
    }).to_string()
}

fn json_output() -> String {
    use serde_json::json;

    json!({
        "parsed_mail": {
            "email_addresses": {
                "from": [{
                    "address": "PIBIeSRqUtiEw1NCg4@fake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "fake.net",
                        "registration_date": "2022-11-18T10:11:12Z",
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regone.zzz",
                        "name": "Reg One",
                    },
                }],
                "links": [],
                "reply_to": [{
                    "address": "blah@possiblynotfake.com",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "possiblynotfake.com",
                        "registration_date": "2022-11-18T10:11:13Z",
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regtwo.zzz",
                        "name": "Reg Two",
                    },
                }],
                "return_path": [{
                    "address": "info@morethanlikelyfake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "morethanlikelyfake.net",
                        "registration_date": "2022-11-18T10:11:14Z",
                    },
                    "registrar": {
                        "name": "Reg Three",
                        "abuse_email_address": "abuse@regthree.zzz",
                    },
                }]
            },
            "links": [],
            "subject": "We’re sorry that we didn’t touch base with you earlier. f309",
        }
    }).to_string()
}

fn setup_mountebank() {
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
}
