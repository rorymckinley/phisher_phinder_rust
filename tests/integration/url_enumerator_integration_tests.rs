use assert_cmd::Command;

use phisher_phinder_rust::mountebank::{clear_all_impostors, setup_head_impostor};

#[test]
fn test_enumerate_fulfillment_nodes() {
    clear_all_impostors();
    setup_head_impostor(4545, true, Some("https://re.direct.to"));

    let mut cmd = Command::cargo_bin("pp-url-enumerator").unwrap();
    cmd.args(["--human"])
        .write_stdin(json_input())
        .assert()
        .success()
        .stdout(predicates::str::contains("https://re.direct.to"));
}

#[test]
fn test_enumerate_fulfillment_nodes_json() {
    clear_all_impostors();
    setup_head_impostor(4545, true, Some("https://re.direct.to"));

    let mut cmd = Command::cargo_bin("pp-url-enumerator").unwrap();
    cmd.write_stdin(json_input())
        .assert()
        .success()
        .stdout(json_output());
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
            "delivery_nodes": [],
            "fulfillment_nodes": [
                {
                    "visible": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "foo.bar",
                            "registration_date": null,
                        },
                        "registrar": null,
                        "url": "http://localhost:4545",
                    },
                    "hidden": null,
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
        },
        "raw_mail": "",
        "reportable_entities": null
    })
    .to_string()
}

fn json_output() -> String {
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
            "delivery_nodes": [],
            "fulfillment_nodes": [
                {
                    "visible": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "foo.bar",
                            "registration_date": null,
                        },
                        "registrar": null,
                        "url": "http://localhost:4545",
                    },
                    "hidden": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "re.direct.to",
                            "registration_date": null,
                        },
                        "registrar": null,
                        "url": "https://re.direct.to",
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
        },
        "raw_mail": "",
        "reportable_entities": null
    })
    .to_string()
}
