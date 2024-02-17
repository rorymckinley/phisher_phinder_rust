use assert_cmd::Command;
use assert_fs::fixture::TempDir;
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
use phisher_phinder_rust::persistence::{
    connect,
    find_runs_for_message_source,
    get_record,
    persist_message_source,
    persist_run,
};
use phisher_phinder_rust::run::Run;
use predicates::prelude::*;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;

const BINARY_NAME: &str = "ppr";

#[test]
fn processes_input_from_stdin() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("PP_DB_PATH", &db_path)
        .write_stdin(multiple_source_input())
        .assert()
        .success();

    assert!(
        lookup_run_linked_to_message(
            &db_path, &sha256(&mail_body_1())
        ).is_some()
    );
    assert!(
        lookup_run_linked_to_message(
            &db_path, &sha256(&mail_body_2())
        ).is_some()
    );
}

#[test]
fn returns_output_from_the_import() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("PP_DB_PATH", &db_path)
        .write_stdin(multiple_source_input())
        .assert()
        .stdout(predicate::str::contains("2 messages processed"))
        .success();
}

#[test]
fn reruns_an_existing_run() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    let conn = connect(&db_path).unwrap();

    let _run_1 = build_run(&conn, 0);
    let run_2 = build_run(&conn, 1);
    let _run_3 = build_run(&conn, 2);

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("PP_DB_PATH", &db_path)
        .args(["--reprocess-run", &format!("{}", run_2.id)])
        .assert()
        .success();

    assert_eq!(
        2,
        find_runs_for_message_source(&conn, &run_2.message_source).len()
    );
}

#[test]
fn fails_if_no_stdin_or_rerun_instruction() {
    let mut cmd = command(BINARY_NAME);

    cmd
        .assert()
        .failure()
        .stderr(predicates::str::contains("message source to STDIN"));
}

#[test]
fn fails_if_no_db_path_provided() {
    let mut cmd = command(BINARY_NAME);

    cmd
        .write_stdin(multiple_source_input())
        .assert()
        .failure()
        .stderr(predicates::str::contains("PP_DB_PATH is a required"));
}

#[test]
fn fails_if_db_cannot_be_opened() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("un/ob/tai/nium");

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("PP_DB_PATH", &db_path)
        .write_stdin(multiple_source_input())
        .assert()
        .failure()
        .stderr(predicates::str::contains("appears to be incorrect"));
}

fn multiple_source_input() -> String {
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

// TODO copy-paste - move this to a utils module?
fn sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    let sha = hasher.finalize();

    sha.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join("")
}

fn lookup_run_linked_to_message(db_path: &Path, hash: &str) -> Option<Run> {
    let conn = connect(db_path).unwrap();

    let message_source = get_record(&conn, hash).unwrap();

    find_runs_for_message_source(&conn, &message_source).pop()
}

fn build_run(conn: &Connection, index: u8) -> Run {
    let persisted_source = persist_message_source(conn, message_source(index));

    let output_data = build_output_data(persisted_source);

    persist_run(conn, &output_data).unwrap()
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

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}
