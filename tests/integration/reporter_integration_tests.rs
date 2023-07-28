use assert_cmd::Command;
use assert_json_diff::assert_json_eq;

#[test]
fn returns_reportable_entities_in_json() {
    let mut cmd = Command::cargo_bin("pp-reporter").unwrap();

    let assert = cmd.write_stdin(json_input()).assert().success();

    let std::process::Output { stdout, .. } = assert.get_output();

    assert_json_eq!(json_output(), String::from_utf8(stdout.to_vec()).unwrap());
}

fn json_input() -> String {
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
    })
    .to_string()
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
        "reportable_entities": {
            "delivery_nodes": [],
            "email_addresses": {
                "from": [],
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
                "reply_to": [],
                "return_path": []
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
        },
        "raw_mail": ""
    })
    .to_string()
}
