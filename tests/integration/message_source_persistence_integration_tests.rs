use assert_cmd::Command;
use assert_cmd::assert::Assert;
use assert_fs::fixture::TempDir;
use assert_json_diff::assert_json_eq;
use fallible_streaming_iterator::FallibleStreamingIterator;
use predicates::prelude::*;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::path::Path;

#[test]
fn stores_message_source_data() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    let mut cmd = Command::cargo_bin("pp-store-mail-source").unwrap();

    cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input())
        .assert()
        .success();

    assert_eq!(2, number_of_entries(&db_path));
}

#[test]
fn returns_message_sources_with_ids() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    let mut cmd = Command::cargo_bin("pp-store-mail-source").unwrap();
    let assert = cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input())
        .assert()
        .success();

    assert_json_output(assert, expected_output());
}

#[test]
fn errors_out_if_no_db_path() {
    let mut cmd = Command::cargo_bin("pp-store-mail-source").unwrap();

    cmd.write_stdin(input())
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

    let mut cmd = Command::cargo_bin("pp-store-mail-source").unwrap();

    cmd.env("PP_DB_PATH", db_path.to_str().unwrap())
        .write_stdin(input())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "PP_DB_PATH ENV variable appears to be incorrect",
        ));
}

fn input() -> String {
    json!([
        {
            "id": null,
            "data": "Message Source 1",
        },
        {
            "id": null,
            "data": "Message Source 2"
        }
    ]).to_string()
}

fn expected_output() -> Value {
    json!([
        {
            "id": 1,
            "data": "Message Source 1",
        },
        {
            "id": 2,
            "data": "Message Source 2"
        }
    ])
}

fn number_of_entries(db_path: &Path) -> usize {
    let conn = Connection::open(db_path).unwrap();

    let mut stmt = conn.prepare("SELECT * FROM message_sources").unwrap();
    let rows = stmt.query([]).unwrap();

    rows.count().unwrap()
}

fn assert_json_output(assert: Assert, expected_output: Value) {
    let json_utf8 = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_utf8).unwrap()).unwrap();

    assert_json_eq!(expected_output, json_data);
}
