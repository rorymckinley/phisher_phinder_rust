use assert_cmd::Command;
use predicates::prelude::*;

use phisher_phinder_rust::mountebank::{
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
            ).and(
                predicates::str::contains("visible.net")
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
            "fulfillment_nodes": [
                {
                    "visible": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "visible.net",
                            "registration_date": null,
                        },
                        "registrar": null,
                        "url": "https://visible.net",
                    },
                    "hidden": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "hidden.com",
                            "registration_date": null,
                        },
                        "registrar": null,
                        "url": "https://hidden.com",
                    }
                }
            ],
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
                "links": [{
                    "address": "perp@alsofake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "alsofake.net",
                        "registration_date": null,
                    },
                    "registrar": null,
                }],
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
                "links": [{
                    "address": "perp@alsofake.net",
                    "domain": {
                        "abuse_email_address": null,
                        "category": "other",
                        "name": "alsofake.net",
                        "registration_date": "2022-11-18T10:11:17Z",
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regsix.zzz",
                        "name": "Reg Six",
                    },
                }],
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
            "fulfillment_nodes": [
                {
                    "visible": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "visible.net",
                            "registration_date":  "2022-11-18T10:11:15Z",
                        },
                        "registrar": {
                            "name": "Reg Four",
                            "abuse_email_address": "abuse@regfour.zzz",
                        },
                        "url": "https://visible.net",
                    },
                    "hidden": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "hidden.com",
                            "registration_date":  "2022-11-18T10:11:16Z",
                        },
                        "registrar": {
                            "name": "Reg Five",
                            "abuse_email_address": "abuse@regfive.zzz",
                        },
                        "url": "https://hidden.com",
                    }
                }
            ],
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
                handle: None,
                registrar: Some("Reg One"),
                abuse_email: Some("abuse@regone.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "possiblynotfake.com",
                handle: None,
                registrar: Some("Reg Two"),
                abuse_email: Some("abuse@regtwo.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 13).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "morethanlikelyfake.net",
                handle: None,
                registrar: Some("Reg Three"),
                abuse_email: Some("abuse@regthree.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 14).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "visible.net",
                handle: None,
                registrar: Some("Reg Four"),
                abuse_email: Some("abuse@regfour.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 15).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "hidden.com",
                handle: None,
                registrar: Some("Reg Five"),
                abuse_email: Some("abuse@regfive.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 16).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "alsofake.net",
                handle: None,
                registrar: Some("Reg Six"),
                abuse_email: Some("abuse@regsix.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()),
                response_code: 200,
            },
        ]
    );
}
