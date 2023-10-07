use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
use phisher_phinder_rust::mountebank::{clear_all_impostors, setup_head_impostor};
use serde_json::json;

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
    let assert = cmd.write_stdin(json_input()).assert().success();

    let json_data = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_data).unwrap()).unwrap();

    assert_json_eq!(json_output(), json_data);
}

fn json_input() -> String {
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
                            "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
                    },
                    "registrar": null,
                }]
            }
        },
        "message_source": {
            "id": 9909,
            "data": "x"
        },
        "reportable_entities": null,
        "run_id": null,
    })
    .to_string()
}

fn json_output() -> serde_json::Value {
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
                            "resolved_domain": null,
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
                            "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
                    },
                    "registrar": null,
                }]
            }
        },
        "message_source": {
            "id": 9909,
            "data": "x"
        },
        "reportable_entities": null,
        "run_id": null,
    })
}
