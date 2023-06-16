use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
use predicates::prelude::*;

use phisher_phinder_rust::mountebank::{
    clear_all_impostors,
    setup_bootstrap_server,
    setup_dns_server,
    setup_ip_v4_server,
    DnsServerConfig,
    IpServerConfig,
};
use chrono::prelude::*;

#[test]
fn test_fetching_rdap_details() {
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

    let assert = cmd
        .write_stdin(json_input())
        .env("RDAP_BOOTSTRAP_HOST", "http://localhost:4545")
        .assert()
        .success();

    let std::process::Output {stdout, ..} = assert.get_output();

    assert_json_eq!(json_output(), String::from_utf8(stdout.to_vec()).unwrap());
}

fn json_input() -> String {
    use serde_json::json;

    json!({
        "parsed_mail": {
            "authentication_results": {
                "dkim": {
                    "result": "Pass",
                    "selector": "ymy",
                    "signature_snippet": "JPh8bpEm",
                    "user_identifier_snippet": "@compromised.zzz",
                },
                "service_identifier": "mx.google.com",
                "spf": {
                    "ip_address": "10.10.10.10",
                    "result": "Pass",
                    "smtp_mailfrom": "info@xxx.fr"
                }
            },
            "delivery_nodes": [
                {
                    "advertised_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "dodgyaf.com",
                            "registration_date": null,
                        },
                        "host": "foo.bar.com",
                        "ip_address": null,
                        "registrar": null,
                    },
                    "observed_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "probablylegit.com",
                            "registration_date": null,
                        },
                        "host": "probablylegit.com",
                        "ip_address": "10.10.10.10",
                        "registrar": null,
                    },
                    "position": 0,
                    "recipient": "mx.google.com",
                    "time": "2022-09-06T23:17:20Z",
                    "trusted": false
                },
            ],
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
        },
        "raw_mail": ""
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
            "authentication_results": {
                "dkim": {
                    "result": "Pass",
                    "selector": "ymy",
                    "signature_snippet": "JPh8bpEm",
                    "user_identifier_snippet": "@compromised.zzz",
                },
                "service_identifier": "mx.google.com",
                "spf": {
                    "ip_address": "10.10.10.10",
                    "result": "Pass",
                    "smtp_mailfrom": "info@xxx.fr"
                }
            },
            "delivery_nodes": [
                {
                    "advertised_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "dodgyaf.com",
                            "registration_date": null,
                        },
                        "host": "foo.bar.com",
                        "infrastructure_provider": null,
                        "ip_address": null,
                        "registrar": null,
                    },
                    "observed_sender": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "probablylegit.com",
                            "registration_date": "2022-11-18T10:11:19Z",
                        },
                        "host": "probablylegit.com",
                        "ip_address": "10.10.10.10",
                        "infrastructure_provider": {
                            "name": "Acme Hosting",
                            "abuse_email_address": "abuse@acmehost.zzz",
                        },
                        "registrar": {
                            "name": "Reg Eight",
                            "abuse_email_address": "abuse@regeight.zzz",
                        },
                    },
                    "position": 0,
                    "recipient": "mx.google.com",
                    "time": "2022-09-06T23:17:20Z",
                    "trusted": false,
                },
            ],
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
        },
        "raw_mail": ""
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
            DnsServerConfig {
                domain_name: "dodgyaf.com",
                handle: None,
                registrar: Some("Reg Seven"),
                abuse_email: Some("abuse@regseven.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 18).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "probablylegit.com",
                handle: None,
                registrar: Some("Reg Eight"),
                abuse_email: Some("abuse@regeight.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 19).unwrap()),
                response_code: 200,
            },
        ]
    );

    setup_ip_v4_server(vec![
        IpServerConfig::response_200(
            "10.10.10.10",
            None,
            ("10.0.0.0", "10.255.255.255"),
            Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")])
        )
    ]);
}
