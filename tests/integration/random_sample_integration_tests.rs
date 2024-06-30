use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use phisher_phinder_rust::data::{EmailAddresses, OutputData, ParsedMail};
use phisher_phinder_rust::persistence::{persist_message_source, persist_run};
use phisher_phinder_rust::message_source::MessageSource;
use predicates::prelude::*;
use rusqlite::Connection;
use std::path::Path;

#[test]
fn returns_a_random_sample_of_persisted_message_source_and_runs() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");

    create_samples(&db_path, 10);

    Command::cargo_bin("pp-display-random-run-results")
        .unwrap()
        .env("PP_DB_PATH", db_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Run ID"));
}

fn create_samples(db_path: &Path, number_of_samples: u8) {
    let conn = Connection::open(db_path).unwrap();

    (0..number_of_samples).for_each(|i| { build_run(&conn, i) })
}

fn build_run(conn: &Connection, index: u8) {
    let persisted_source = persist_message_source(conn, &message_source(index));

    let output_data = build_output_data(persisted_source);

    persist_run(conn, &output_data).unwrap();
}

fn message_source(id: u8) -> MessageSource {
    MessageSource::new(&format!("src #{id}"))
}

fn build_output_data(message_source: MessageSource) -> OutputData {
    OutputData::new(parsed_mail(), message_source)
}

fn parsed_mail() -> ParsedMail {
    ParsedMail::new(None, vec![], email_addresses(), vec![], None)
}

fn email_addresses() -> EmailAddresses {
    EmailAddresses {
        from: vec![],
        links: vec![],
        reply_to: vec![],
        return_path: vec![]
    }
}
