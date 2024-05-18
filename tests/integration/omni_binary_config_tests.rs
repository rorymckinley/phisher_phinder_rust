use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use predicates::prelude::*;
use serde::Serialize;
use std::path::Path;

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
        .args(["config", "location"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_path.to_str().unwrap()));
}

#[test]
fn lists_current_config_contents() {
    let temp = TempDir::new().unwrap();

    store_config(temp.path());

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("abuse_notifications_author_name: Fred Flintstone")
        );
}

#[test]
fn sets_config() {
    let temp = TempDir::new().unwrap();

    store_config(temp.path());

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["config", "set", "--abuse-notifications-author-name", "Barney Rubble"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("abuse_notifications_author_name: Barney Rubble")
        );
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}

fn store_config(path: &Path) {
    #[derive(Serialize)]
    struct TestConfig<'a> {
        abuse_notifications_author_name: &'a str,
    }

    let mut target_path = path.to_path_buf();
    target_path.push(".config");
    target_path.push("phisher_eagle");
    target_path.push("default-config.toml");

    let config = TestConfig { abuse_notifications_author_name: "Fred Flintstone" };

    confy::store_path(&target_path, config).unwrap();
}
