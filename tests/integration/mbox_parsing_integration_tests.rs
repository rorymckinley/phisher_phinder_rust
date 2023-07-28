use assert_cmd::Command;
use assert_json_diff::assert_json_eq;
use serde_json::{json, Value};

#[test]
fn parses_mbox_file_test() {
    let mut cmd = Command::cargo_bin("pp-mbox-parser").unwrap();

    let assert = cmd.write_stdin(input()).assert().success();
    let json_utf8 = &assert.get_output().stdout;

    let json_data: serde_json::Value =
        serde_json::from_str(std::str::from_utf8(json_utf8).unwrap()).unwrap();

    assert_json_eq!(expected_json(), json_data);
}

fn input() -> String {
    format!("{}\r\n{}", entry_1(), entry_2())
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

fn expected_json() -> Value {
    json!([mail_body_1(), mail_body_2()])
}
