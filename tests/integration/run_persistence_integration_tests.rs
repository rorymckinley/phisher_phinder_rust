use assert_cmd::assert::Assert;
use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use assert_json_diff::assert_json_eq;
use predicates::prelude::*;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const BINARY_NAME: &str = "pp-store-run-details";

#[test]
fn persists_run_details() {
    let message_source_data = "x";

    let temp = TempDir::new().unwrap();

    let db_path: PathBuf = build_db_path(temp.path());

    persist_message_source(&db_path, message_source_data);

    let conn = db_connection(&db_path);

    let message_source_id = get_message_source_id(&conn);

    let mut cmd = command(BINARY_NAME);

    cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input(message_source_id, message_source_data))
        .assert()
        .success();

    assert!(has_run_record(&conn, message_source_id));
}

#[test]
fn outputs_data_in_json_format_including_run_id() {
    let message_source_data = "x";

    let temp = TempDir::new().unwrap();

    let db_path: PathBuf = build_db_path(temp.path());

    persist_message_source(&db_path, message_source_data);

    let conn = db_connection(&db_path);

    let message_source_id = get_message_source_id(&conn);

    let mut cmd = command(BINARY_NAME);

    let assert = cmd
        .env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input(message_source_id, message_source_data))
        .assert()
        .success();

    assert_json_output(
        assert,
        expected_json_output(message_source_id, message_source_data),
    );
}

#[test]
fn errors_out_if_no_db_path() {
    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();

    cmd.write_stdin(input(999, "does-not-matter"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "PP_DB_PATH ENV variable is required",
        ));
}

#[test]
fn errors_out_if_db_path_is_bad() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("/unobtainium/pp.sqlite3");

    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();

    cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input(999, "does-not-matter"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "PP_DB_PATH ENV variable appears to be incorrect",
        ));
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}

fn build_db_path(root_path: &Path) -> PathBuf {
    root_path.join("pp.sqlite3")
}

fn persist_message_source(db_path: &Path, data: &str) {
    let input = json!([{"id": null, "data": data}]).to_string();

    let mut cmd = Command::cargo_bin("pp-store-mail-source").unwrap();

    cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input)
        .assert()
        .success();
}

fn db_connection(db_path: &Path) -> Connection {
    Connection::open(db_path).unwrap()
}

fn get_message_source_id(conn: &Connection) -> u32 {
    let mut stmt = conn.prepare("SELECT id FROM message_sources").unwrap();

    stmt.query_row([], |row| row.get::<usize, u32>(0)).unwrap()
}

fn input(message_source_id: u32, message_source_data: &str) -> String {
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                            "resolved_domain": null,
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
                            "resolved_domain": null,
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
                            "resolved_domain": null,
                        },
                        "registrar": {
                            "name": "Reg Four",
                            "abuse_email_address": "abuse@regfour.zzz",
                        },
                        "url": "https://visible.net",
                    },
                    "investigable": true,
                    "hidden": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "hidden.com",
                            "registration_date":  "2022-11-18T10:11:16Z",
                            "resolved_domain": null,
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
                        "resolved_domain": null,
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regsix.zzz",
                        "name": "Reg Six",
                    },
                }],
                "reply_to": [],
                "return_path": []
            },
            "fulfillment_nodes_container": {
                "duplicates_removed": false,
                "nodes": [
                    {
                        "visible": {
                            "domain": {
                                "abuse_email_address": null,
                                "category": "other",
                                "name": "visible.net",
                                "registration_date":  "2022-11-18T10:11:15Z",
                                "resolved_domain": null,
                            },
                            "registrar": {
                                "name": "Reg Four",
                                "abuse_email_address": "abuse@regfour.zzz",
                            },
                            "url": "https://visible.net",
                        },
                        "investigable": true,
                        "hidden": {
                            "domain": {
                                "abuse_email_address": null,
                                "category": "other",
                                "name": "hidden.com",
                                "registration_date":  "2022-11-18T10:11:16Z",
                                "resolved_domain": null,
                            },
                            "registrar": {
                                "name": "Reg Five",
                                "abuse_email_address": "abuse@regfive.zzz",
                            },
                            "url": "https://hidden.com",
                        }
                    }
                ]
            }
        },
        "message_source": {
            "id": message_source_id,
            "data": message_source_data
        },
        "notifications": []
    })
    .to_string()
}

fn expected_json_output(message_source_id: u32, message_source_data: &str) -> Value {
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                        "resolved_domain": null,
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
                            "resolved_domain": null,
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
                            "resolved_domain": null,
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
                            "resolved_domain": null,
                        },
                        "registrar": {
                            "name": "Reg Four",
                            "abuse_email_address": "abuse@regfour.zzz",
                        },
                        "url": "https://visible.net",
                    },
                    "investigable": true,
                    "hidden": {
                        "domain": {
                            "abuse_email_address": null,
                            "category": "other",
                            "name": "hidden.com",
                            "registration_date":  "2022-11-18T10:11:16Z",
                            "resolved_domain": null,
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
                        "resolved_domain": null,
                    },
                    "registrar": {
                        "abuse_email_address": "abuse@regsix.zzz",
                        "name": "Reg Six",
                    },
                }],
                "reply_to": [],
                "return_path": []
            },
            "fulfillment_nodes_container": {
                "duplicates_removed": false,
                "nodes": [
                    {
                        "visible": {
                            "domain": {
                                "abuse_email_address": null,
                                "category": "other",
                                "name": "visible.net",
                                "registration_date":  "2022-11-18T10:11:15Z",
                                "resolved_domain": null,
                            },
                            "registrar": {
                                "name": "Reg Four",
                                "abuse_email_address": "abuse@regfour.zzz",
                            },
                            "url": "https://visible.net",
                        },
                        "investigable": true,
                        "hidden": {
                            "domain": {
                                "abuse_email_address": null,
                                "category": "other",
                                "name": "hidden.com",
                                "registration_date":  "2022-11-18T10:11:16Z",
                                "resolved_domain": null,
                            },
                            "registrar": {
                                "name": "Reg Five",
                                "abuse_email_address": "abuse@regfive.zzz",
                            },
                            "url": "https://hidden.com",
                        }
                    }
                ]
            }
        },
        "message_source": {
            "id": message_source_id,
            "data": message_source_data
        },
        "notifications": [],
        "run_id": 1
    })
}

fn has_run_record(conn: &Connection, message_source_id: u32) -> bool {
    let mut stmt = conn
        .prepare("SELECT id FROM runs where message_source_id = ?")
        .unwrap();

    let row_result = stmt.query_row([message_source_id], |row| row.get::<usize, u32>(0));

    matches!(row_result, Ok(_))
}

fn assert_json_output(assert: Assert, expected_output: Value) {
    let json_utf8 = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_utf8).unwrap()).unwrap();

    assert_json_eq!(expected_output, json_data);
}
