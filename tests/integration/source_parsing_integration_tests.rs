use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
use serde_json::{json, Value};

const BINARY_NAME: &str ="pp-source-parser";

#[test]
fn parses_mbox_file_test() {
    let mut cmd = command(BINARY_NAME);

    let assert = cmd.write_stdin(multiple_source_input()).assert().success();
    let json_utf8 = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_utf8).unwrap()).unwrap();

    assert_json_eq!(expected_multiple_source_json(), json_data);
}

#[test]
fn parses_single_mail_source_file() {
    let mut cmd = command(BINARY_NAME);

    let assert = cmd.write_stdin(single_source_input()).assert().success();
    let json_utf8 = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_utf8).unwrap()).unwrap();

    assert_json_eq!(expected_single_source_json(), json_data);
}

fn multiple_source_input() -> String {
    format!("{}\r\n{}", entry_1(), entry_2())
}

fn single_source_input() -> String {
    mail_body_1()
}

fn entry_1() -> String {
    format!(
        "From 123@xxx Sun Jun 11 20:53:34 +0000 2023\r\n{}",
        mail_body_1()
    )
}

fn entry_2() -> String {
    format!(
        "From 456@xxx Sun Jun 11 20:53:35 +0000 2023\r\n{}",
        mail_body_2()
    )
}

fn mail_body_1() -> String {
    "Delivered-To: victim1@test.zzz\r
Subject: Dodgy Subject 1"
        .into()
}

fn mail_body_2() -> String {
    "Delivered-To: victim2@test.zzz\r
Subject: Dodgy Subject 2"
        .into()
}

fn expected_multiple_source_json() -> Value {
    json!([mail_body_1(), mail_body_2()])
}

fn expected_single_source_json() -> Value {
    json!([mail_body_1()])
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}
