use assert_cmd::Command;
use assert_fs::TempDir;
use phisher_phinder_rust::authentication_results::AuthenticationResults;
use phisher_phinder_rust::data::{
    EmailAddressData,
    EmailAddresses,
    FulfillmentNodesContainer,
    OutputData,
    ParsedMail,
    ReportableEntities
};
use phisher_phinder_rust::message_source::MessageSource;
use phisher_phinder_rust::persistence::{persist_message_source, persist_run};
use predicates::prelude::*;
use rusqlite::Connection;

#[test]
fn returns_the_message_source_for_specified_run_id() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args(["--message-source", &format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("src 1"));
}

#[test]
fn does_return_message_source_if_no_message_source_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args([&format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("src 1").not());
}

#[test]
fn returns_message_source_suitable_for_piping_if_pipe_message_source_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args(["--pipe-message-source", &format!("{run_id}")])
        .assert()
        .success()
        .stdout("src 1");
}

#[test]
fn returns_email_addresses_if_email_addresses_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args(["--email-addresses", &format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("from_1@test.com"));
}

#[test]
fn returns_no_email_address_if_no_email_addresses_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args([&format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("from_1@test.com").not());
}

#[test]
fn returns_authentication_results_if_authentication_results_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args(["--authentication-results", &format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("mx.google.com"));
}

#[test]
fn returns_no_authentication_results_if_no_authentication_results_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args([&format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("mx.google.com").not());
}

#[test]
fn returns_reportable_entities_if_reportable_entities_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args(["--reportable-entities", &format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("reportable@test.com"));
}

#[test]
fn returns_reportable_entities_if_no_reportable_entities_flag() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let run_id = build_run(&conn, 1);

    Command::cargo_bin("pp-fetch-run-details")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args([&format!("{run_id}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("reportable@test.com").not());
}

fn build_run(conn: &Connection, index: u8) -> i64 {
    let persisted_source = persist_message_source(conn, message_source(index));

    let output_data = build_output_data(persisted_source);

    persist_run(conn, &output_data).unwrap().id.into()
}

fn message_source(index: u8) -> MessageSource {
    MessageSource::new(&format!("src {index}"))
}

fn build_output_data(message_source: MessageSource) -> OutputData {
    OutputData {
        message_source,
        notifications: vec![],
        parsed_mail: parsed_mail(),
        reportable_entities: Some(reportable_entities()),
        run_id: None
    }
}

fn parsed_mail() -> ParsedMail {
    ParsedMail::new(
        Some(authentication_results()),
        vec![],
        email_addresses("from_1@test.com"),
        vec![],
        None
    )
}

fn authentication_results() -> AuthenticationResults {
    AuthenticationResults {
        dkim: None,
        service_identifier: Some("mx.google.com".into()),
        spf: None,
    }
}

fn email_addresses(email_address: &str) -> EmailAddresses {
    EmailAddresses {
        from: vec![EmailAddressData::from_email_address(email_address)],
        links: vec![],
        reply_to: vec![],
        return_path: vec![]
    }
}

fn reportable_entities() -> ReportableEntities {
    ReportableEntities {
        delivery_nodes: vec![],
        email_addresses: email_addresses("reportable@test.com"),
        fulfillment_nodes_container: FulfillmentNodesContainer {
            duplicates_removed: false,
            nodes: vec![],
        }
    }
}
