use assert_cmd::Command;
use serde_json::json;

const BINARY_NAME: &str = "pp-source-splitter";

#[test]
fn splits_serialised_array_into_individual_source_chunks() {
    let mut cmd = Command::cargo_bin(BINARY_NAME).unwrap();
    cmd.write_stdin(input())
        .assert()
        .success()
        .stdout(expected_output());
}

fn input() -> String {
    json!([
        {"id": 1, "data": "x"},
        {"id": 2, "data": "y"},
        {"id": 3, "data": "z"},
    ])
    .to_string()
}

fn expected_output() -> String {
    format!(
        "{}\n{}\n{}\n",
        json!({"data": "x", "id": 1}),
        json!({"data": "y", "id": 2}),
        json!({"data": "z", "id": 3}),
    )
}
