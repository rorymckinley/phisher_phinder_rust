use assert_cmd::Command;
use assert_fs::fixture::TempDir;
use chrono::prelude::*;
use phisher_phinder_rust::authentication_results::AuthenticationResults;
use phisher_phinder_rust::data::{
    EmailAddressData,
    EmailAddresses,
    FulfillmentNodesContainer,
    OutputData,
    ParsedMail,
    ReportableEntities
};
use phisher_phinder_rust::mail_trap::MailTrap;
use phisher_phinder_rust::message_source::MessageSource;
use phisher_phinder_rust::mountebank::{
    DnsServerConfig,
    clear_all_impostors,
    setup_dns_server,
    setup_bootstrap_server
};
use phisher_phinder_rust::persistence::{
    connect,
    find_runs_for_message_source,
    get_record,
    persist_message_source,
    persist_run,
};
use phisher_phinder_rust::run::Run;
use phisher_phinder_rust::service_configuration::FileConfig;
use predicates::prelude::*;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;

const BINARY_NAME: &str = "ppr";

#[test]
fn processes_input_from_stdin() {
    clear_all_impostors();
    setup_bootstrap_server();

    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");

    build_config_file(&config_file_path, Some(&db_path));

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process"])
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
    clear_all_impostors();
    setup_bootstrap_server();

    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");

    build_config_file(&config_file_path, Some(&db_path));

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process"])
        .write_stdin(multiple_source_input())
        .assert()
        .stdout(predicate::str::contains("2 messages processed"))
        .success();
}

#[test]
fn reruns_an_existing_run() {
    clear_all_impostors();
    setup_bootstrap_server();

    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");

    build_config_file(&config_file_path, Some(&db_path));

    let conn = connect(&db_path).unwrap();

    let _run_1 = build_run(&conn, 0);
    let run_2 = build_run(&conn, 1);
    let _run_3 = build_run(&conn, 2);

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process", "--reprocess-run", &format!("{}", run_2.id)])
        .assert()
        .success();

    assert_eq!(
        2,
        find_runs_for_message_source(&conn, &run_2.message_source).len()
    );
}

#[test]
fn fails_if_no_stdin_or_rerun_instruction() {
    let temp = TempDir::new().unwrap();
    let mut cmd = command(BINARY_NAME);

    cmd
        .args(["process"])
        .env("HOME", temp.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Please pass in message source"));
}

#[test]
fn fails_if_no_db_path_provided() {
    let temp = TempDir::new().unwrap();
    let mut cmd = command(BINARY_NAME);
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");

    build_config_file(&config_file_path, None);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process"])
        .write_stdin(multiple_source_input())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Please configure db_path"));
}

#[test]
fn fails_if_db_cannot_be_opened() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("un/ob/tai/nium");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");

    build_config_file(&config_file_path, Some(&db_path));

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process"])
        .write_stdin(multiple_source_input())
        .assert()
        .failure()
        .stderr(predicates::str::contains("appears to be incorrect"));
}

#[test]
fn sends_mail_for_any_reportable_entities() {
    clear_all_impostors();
    setup_bootstrap_server();
    initialise_mail_trap();

    setup_dns_server(vec![
        DnsServerConfig {
            domain_name: "fake.net",
            handle: None,
            registrar: Some("Reg One"),
            abuse_email: Some("abuse@regone.zzz"),
            registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
            response_code: 200,
        },
    ]);
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");
    let mail_trap = initialise_mail_trap();

    build_config_file(&config_file_path, Some(&db_path));

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args(["process", "--send-abuse-notifications"])
        .write_stdin(multiple_source_input())
        .assert()
        .success();

    let email = mail_trap.get_last_email();

    assert_eq!(email.to, Some("abuse@regone.zzz".into()));
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
Subject: Dodgy Subject 1\r
Content-Type: text/html\r\n\r
<a href=\"https://foo.bar/baz\">Click Me</a>".into()
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

fn build_config_file(config_file_location: &Path, db_path: Option<&Path>) {
    let config = FileConfig {
        abuse_notifications_author_name: Some("Phisher Eagle".into()),
        abuse_notifications_from_address: Some("security@phisher_eagle.com".into()),
        db_path: db_path.map(|path| path.to_str().unwrap().into()),
        rdap_bootstrap_host: Some("http://localhost:4545".into()),
        smtp_host_uri: std::env::var("PP_SMTP_HOST_URI").ok(),
        smtp_password: std::env::var("PP_SMTP_PASSWORD").ok(),
        smtp_username: std::env::var("PP_SMTP_USERNAME").ok(),
        ..FileConfig::default()
    };

    confy::store_path(config_file_location, config).unwrap();
}

fn initialise_mail_trap() -> MailTrap {
    let mail_trap = MailTrap::new(mail_trap_api_token());

    mail_trap.clear_mails();

    mail_trap
}

fn mail_trap_api_token() -> String {
    std::env::var("MAILTRAP_API_TOKEN").unwrap()
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}
