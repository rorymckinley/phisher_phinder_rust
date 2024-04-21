use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use predicates::prelude::*;

const BINARY_NAME: &str = "ppr";

#[test]
fn shows_path_to_config_file() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();
    let mut expected_path = temp_path.to_path_buf();
    expected_path.push(".config");
    expected_path.push("phisher_eagle");
    expected_path.push("default-config.toml");

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["config", "--location"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_path.to_str().unwrap()));
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}
