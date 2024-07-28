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
    setup_bootstrap_server,
    setup_dns_server,
    setup_head_impostor,
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
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");
    let db_path = temp.path().join("pp.sqlite3");
    let mut cmd = command(BINARY_NAME);

    build_config_file(&config_file_path, Some(&db_path));

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
        .stderr(predicates::str::contains("Invalid configuration: Please configure db_path"));
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
#[ignore]
fn sends_mail_for_any_reportable_entities() {
    setup_mountebank();
    initialise_mail_trap();

    let mut expected_recipients: Vec<String> = vec![
        String::from("abuse@regone.zzz"),
        String::from("abuse@regtwo.zzz"),
    ];
    expected_recipients.sort();

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

    assert_eq!(mail_trap_recipients(&mail_trap), expected_recipients);
}

#[test]
#[ignore]
fn sends_mail_to_specified_test_recipient() {
    setup_mountebank();
    initialise_mail_trap();

    let expected_recipients: Vec<String> = vec![
        String::from("recipient@phishereagle.com"),
        String::from("recipient@phishereagle.com"),
    ];

    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("pp.sqlite3");
    let config_file_path = temp.path().join(".config/phisher_eagle/default-config.toml");
    let mail_trap = initialise_mail_trap();

    build_config_file(&config_file_path, Some(&db_path));

    let mut cmd = command(BINARY_NAME);

    cmd
        .env("HOME", temp.path().to_str().unwrap())
        .args([
            "process",
            "--send-abuse-notifications",
            "--test-recipient", "recipient@phishereagle.com"
        ])
        .write_stdin(multiple_source_input())
        .assert()
        .success();

    assert_eq!(mail_trap_recipients(&mail_trap), expected_recipients);
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
    format!(
        "{}\r\n{}\r\n{}\r\n\r\n{}\r\n{}",
        "Delivered-To: victim1@test.zzz",
        "Subject: Dodgy Subject 1",
        "Content-Type: text/html",
        "<a href=\"http://localhost:4560\">Click Me</a>",
        "<a href=\"http://localhost:4562\">Click Me</a>",
    )
}

fn mail_body_2() -> String {
    format!(
        "{}\r\n{}\r\n{}\r\n\r\n{}\r\n{}",
        "Delivered-To: victim1@test.zzz",
        "Subject: Dodgy Subject 2",
        "Content-Type: text/html",
        "<a href=\"http://localhost:4561\">Click Me</a>",
        "<a href=\"http://localhost:4563\">Click Me</a>",
    )
}

fn setup_mountebank() {
    clear_all_impostors();
    setup_bootstrap_server();

    setup_head_impostor(4560, true, Some("http://re.directone.net"));
    setup_head_impostor(4561, true, Some("http://re.directtwo.net"));

    setup_dns_server(vec![
        DnsServerConfig {
            domain_name: "re.directone.net",
            handle: None,
            registrar: Some("Reg One"),
            abuse_email: Some("abuse@regone.zzz"),
            registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
            response_code: 200,
        },
        DnsServerConfig {
            domain_name: "re.directtwo.net",
            handle: None,
            registrar: Some("Reg Six"),
            abuse_email: Some("abuse@regtwo.zzz"),
            registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()),
            response_code: 200,
        },
    ]);
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
    let persisted_source = persist_message_source(conn, &message_source(index));

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
        smtp_host_uri: std::env::var("TEST_SMTP_URI").ok(),
        smtp_password: std::env::var("TEST_SMTP_PASSWORD").ok(),
        smtp_username: std::env::var("TEST_SMTP_USERNAME").ok(),
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

fn mail_trap_recipients(mail_trap: &MailTrap) -> Vec<String> {
    let mut mail_recipients: Vec<String> = mail_trap
        .get_all_emails()
        .into_iter()
        .map(|email| email.to.unwrap())
        .collect();

    mail_recipients.sort();

    mail_recipients
}

fn command(binary_name: &str) -> Command {
    Command::cargo_bin(binary_name).unwrap()
}
