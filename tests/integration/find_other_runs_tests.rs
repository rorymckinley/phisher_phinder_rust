use assert_cmd::Command;
use assert_fs::TempDir;
use phisher_phinder_rust::data::{
    EmailAddressData,
    EmailAddresses,
    OutputData,
    ParsedMail
};
use phisher_phinder_rust::message_source::MessageSource;
use phisher_phinder_rust::persistence::{persist_message_source, persist_run};
use predicates::prelude::*;
use rusqlite::Connection;

#[test]
fn returns_ids_of_runs_that_share_message_source() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let conn = Connection::open(&db_path).unwrap();

    let target_run_id = build_run(&conn, 1);
    let other_run_id = build_run(&conn, 1);
    let other_message_run_id = build_run(&conn, 2);

    Command::cargo_bin("pp-find-other-runs")
        .unwrap()
        .env("PP_DB_PATH", &db_path)
        .args([&format!("{target_run_id}")])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("{target_run_id}")).and(
                predicate::str::contains(format!("{other_run_id}"))
            ).and(
                predicate::str::contains(format!("{other_message_run_id}")).not()
            )
        );
}

fn build_run(conn: &Connection, index: u8) -> i64 {
    let persisted_source = persist_message_source(conn, message_source(index));

    let output_data = build_output_data(persisted_source);

    persist_run(conn, &output_data).unwrap()
}

fn message_source(index: u8) -> MessageSource {
    MessageSource::new(&format!("src {index}"))
}

fn build_output_data(message_source: MessageSource) -> OutputData {
    OutputData::new(parsed_mail(), message_source)
}

fn parsed_mail() -> ParsedMail {
    ParsedMail::new(
        None,
        vec![],
        email_addresses("from_1@test.com"),
        vec![],
        None
    )
}

fn email_addresses(email_address: &str) -> EmailAddresses {
    EmailAddresses {
        from: vec![EmailAddressData::from_email_address(email_address)],
        links: vec![],
        reply_to: vec![],
        return_path: vec![]
    }
}
