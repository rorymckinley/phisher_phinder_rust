use crate::data::OutputData;
use crate::enumerator::enumerate;
use crate::errors::AppError;
use crate::notification::add_notifications;
use crate::persistence::persist_run;
use crate::populator::populate;
use crate::reporter::add_reportable_entities;
use crate::run::Run;
use crate::run_presenter::present;
use crate::service_configuration;
use rusqlite::Connection;
use std::future::Future;
use std::sync::Arc;
use test_friendly_rdap_client::Client;
use tokio::task::JoinError;

mod configuration;
mod message_source;
mod persistence;
mod analysis;

pub async fn execute_command<T>(config: &T) -> Result<String, AppError>
where T: service_configuration::Configuration {
    let command_config_two = configuration::extract_command_config(config)?;

    let input_data: Vec<OutputData> = command_config_two
        .inputs
        .iter()
        .map(|message_source| {
            let persisted_source = persistence::persist_message_source(
                &command_config_two.db_connection,
                message_source
            );

            // TODO Test error handling
            analysis::analyse_message_source(
                persisted_source,
                command_config_two.trusted_recipient
            )
            .unwrap()
        })
        .collect();

    // From https://users.rust-lang.org/t/how-to-tokio-join-on-a-vector-of-futures/73233
    let enumeration_tasks: Vec<_> = input_data
        .into_iter()
        .map(|mail_data| { tokio::spawn(enumerate(mail_data)) })
        .collect();

    let mut client = Client::new();

    if let Some(bootstrap_host) = config.rdap_bootstrap_host() {
        client.set_base_bootstrap_url(bootstrap_host)
    }

    let mut records: Vec<OutputData> = vec![];

    for task in enumeration_tasks {
        records.push(task.await.unwrap())
    }

    let bootstrap = client.fetch_bootstrap().await.unwrap();

    let b_strap = Arc::new(bootstrap);

    let populate_tasks: Vec<_> = records
        .into_iter()
        .map(|task| { tokio::spawn(populate(Arc::clone(&b_strap), task))})
        .collect();

    let records_with_reportable_entities = generate_reportable_entities(populate_tasks).await;

    let records_with_notifications = add_notifications_for_records(
        records_with_reportable_entities
    );

    let run_result = persist_runs(&command_config_two.db_connection, records_with_notifications)?;

    // TODO The error in the Result is a tuple of (Connection, Error)
    // Add error conversion for this
    command_config_two.db_connection.close().unwrap();

    match run_result {
        RunStorageResult::MultipleRuns(count) => Ok(format!("{count} messages processed.")),
        RunStorageResult::SingleRun(boxed_run) => {
            present(*boxed_run, config)
        }
    }
}

fn persist_runs(connection: &Connection, output_data_records: Vec<OutputData>)
    -> Result<RunStorageResult, AppError> {
    let mut runs: Vec<Run> = vec![];

    for record in output_data_records {
        // TODO Better error handling here -  what do we do if enumerating output data
        // fails?
        // TODO what do we if enumerating works out but we get a JoinError from the `.await`
        // How do I test that?
        let run = persist_run(connection, &record)?;
        runs.push(run);
    }


    let run_count = runs.len();

    // TODO Cover the empty use case
    if run_count == 1 {
        //TODO Better error handling
        Ok(RunStorageResult::SingleRun(Box::new(runs.pop().unwrap())))
    } else {
        Ok(RunStorageResult::MultipleRuns(run_count))
    }
}

async fn generate_reportable_entities(
    output_tasks: Vec<impl Future<Output=Result<OutputData, JoinError>>>
) -> Vec<OutputData> {
    let mut output: Vec<OutputData> = vec![];

    for task in output_tasks {
        // TODO Better error handling here -  what do we do if enumerating output data
        // fails?
        // TODO what do we if enumerating works out but we get a JoinError from the `.await`
        // How do I test that?
        output.push(add_reportable_entities(task.await.unwrap()))
    }

    output
}

fn add_notifications_for_records(records: Vec<OutputData>) -> Vec<OutputData> {
    records
        .into_iter()
        .map(add_notifications)
        .collect()
}

enum RunStorageResult {
    MultipleRuns(usize),
    SingleRun(Box<Run>),
}

#[cfg(test)]
mod process_message_execute_command_from_stdin_tests {
    use assert_fs::fixture::TempDir;
    use crate::message_source::MessageSource;
    use crate::mountebank::{clear_all_impostors, setup_bootstrap_server};
    use crate::persistence::{connect, find_runs_for_message_source, get_record};
    use crate::run::Run;
    use std::path::Path;
    use support::{build_cli, build_config, build_config_file, build_config_location, sha256};

    use super::*;

    #[test]
    fn creates_messages_sources_provided_via_input() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(Some(&input), &cli, &config_file_location);

        let result = tokio_test::block_on(execute_command(&config));

        assert!(result.is_ok());

        let persisted_hashes = lookup_message_source_hashes(&db_path);

        assert!(persisted_hashes.contains(&sha256(&mail_body_1())));
        assert!(persisted_hashes.contains(&sha256(&mail_body_2())));
    }

    #[test]
    fn persists_a_run_for_each_message_source() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(Some(&input), &cli, &config_file_location);

        let _ = tokio_test::block_on(execute_command(&config));

        let run_1_result = lookup_run_linked_to_message(&db_path, &sha256(&mail_body_1()));
        assert!(run_1_result.is_some());
        let run_1_data_source = run_1_result.unwrap().data.message_source;
        assert_eq!(MessageSource::persisted_record(1, &mail_body_1()), run_1_data_source);

        let run_2_result = lookup_run_linked_to_message(&db_path, &sha256(&mail_body_2()));
        assert!(run_2_result.is_some());
        let run_2_data_source = run_2_result.unwrap().data.message_source;
        assert_eq!(MessageSource::persisted_record(2, &mail_body_2()), run_2_data_source);
    }

    #[test]
    fn returns_number_of_messages_processed_if_multiple_messages() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(Some(&input), &cli, &config_file_location);

        let output = tokio_test::block_on(execute_command(&config)).unwrap();

        assert_eq!(String::from("2 messages processed."), output);
    }

    #[test]
    fn returns_the_run_details_if_single_message() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");
        let input = single_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(Some(&input), &cli, &config_file_location);

        let output = tokio_test::block_on(execute_command(&config)).unwrap();

        assert!(output.contains("Abuse Email Address"));
        assert!(output.contains(""))
    }

    fn multiple_source_input() -> String {
        format!("{}\r\n{}", entry_1(), entry_2())
    }

    fn single_source_input() -> String {
        entry_1()
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
            "{}\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 1"
        )
    }

    fn mail_body_2() -> String {
        format!(
            "{}\r\n{}",
            "Delivered-To: victim2@test.zzz",
            "Subject: Dodgy Subject 2"
        )
    }

    fn lookup_message_source_hashes(db_path: &Path) -> Vec<String> {
        let conn = connect(db_path).unwrap();

        let mut stmt = conn
            .prepare("SELECT hash FROM message_sources")
            .unwrap();

        let rows = stmt
            .query_map([], |row| row.get(0))
            .unwrap();

        rows
            .map(|row_result| row_result.unwrap())
            .collect()
    }

    fn lookup_run_linked_to_message(db_path: &Path, hash: &str) -> Option<Run> {
        let conn = connect(db_path).unwrap();

        let message_source = get_record(&conn, hash).unwrap();

        find_runs_for_message_source(&conn, &message_source).pop()
    }
}

#[cfg(test)]
mod process_message_execute_command_rerun_tests {
    use assert_fs::fixture::TempDir;
    use crate::authentication_results::AuthenticationResults;
    use crate::data::{
        EmailAddressData,
        EmailAddresses,
        FulfillmentNodesContainer,
        ParsedMail,
        ReportableEntities
    };
    use crate::message_source::MessageSource;
    use crate::mountebank::{clear_all_impostors, setup_bootstrap_server};
    use crate::persistence::{
        connect,
        find_run,
        find_runs_for_message_source,
        persist_message_source
    };
    use rusqlite::Connection;
    use support::{build_cli, build_config, build_config_file, build_config_location};

    use super::*;

    #[test]
    fn reruns_an_existing_run() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let _run_1_id = build_run(&conn, 0);
        let run_2_id = build_run(&conn, 1);
        let _run_3_id = build_run(&conn, 2);

        let cli = build_cli(Some(run_2_id));

        build_config_file(&config_file_location, &db_path);

        let config = build_config(None, &cli, &config_file_location);

        let result = tokio_test::block_on(execute_command(&config));

        assert!(result.is_ok());

        let run_2 = find_run(&conn, run_2_id).unwrap();

        assert_eq!(
            2,
            find_runs_for_message_source(&conn, &run_2.message_source).len()
        );
    }

    #[test]
    fn correctly_persists_message_source_in_new_run_data() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let _run_1_id = build_run(&conn, 0);
        let run_2_id = build_run(&conn, 1);
        let run_3_id = build_run(&conn, 2);

        let cli = build_cli(Some(run_2_id));

        build_config_file(&config_file_location, &db_path);

        let config = build_config(None, &cli, &config_file_location);

        let _ = tokio_test::block_on(execute_command(&config));

        let run_2 = find_run(&conn, run_2_id).unwrap();

        let run_2_rerun = find_run(&conn, run_3_id + 1).unwrap();

        assert_eq!(run_2.data.message_source, run_2_rerun.data.message_source);
    }

    #[test]
    fn raises_error_if_run_does_not_exist() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let run_id = build_run(&conn, 0);

        let cli = build_cli(Some(run_id + 100));

        build_config_file(&config_file_location, &db_path);

        let config = build_config(None, &cli, &config_file_location);

        match tokio_test::block_on(execute_command(&config)) {
            Err(AppError::SpecifiedRunMissing) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    fn build_run(conn: &Connection, index: u8) -> i64 {
        let persisted_source = persist_message_source(conn, &message_source(index));

        let output_data = build_output_data(persisted_source);

        persist_run(conn, &output_data).unwrap().id.into()
    }

    fn message_source(index: u8) -> MessageSource {
        MessageSource::new(&format!("src {index}"))
    }

    fn build_output_data(message_source: MessageSource) -> OutputData {
        OutputData {
            message_source,
            parsed_mail: parsed_mail(),
            notifications: vec![],
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
}

#[cfg(test)]
mod process_message_execute_command_common_errors_tests {
    use assert_fs::fixture::TempDir;
    use crate::cli::{ProcessArgs, SingleCli, SingleCliCommands};
    use crate::service_configuration::ServiceConfiguration;
    use support::{build_config_file, build_config_location};

    use super::*;

    #[test]
    fn returns_err_if_db_cannot_be_initialised() {
        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("un/ob/tai/nium");

        let cli = build_cli(None);

        build_config_file(&config_file_location, &db_path);

        let config = ServiceConfiguration::new(
            Some("message_source"),
            &cli,
            &config_file_location
        ).unwrap();

        let result = tokio_test::block_on(execute_command(&config));

        match result {
            Err(AppError::DatabasePathIncorrect(path)) => {
                assert_eq!(db_path.to_str().unwrap(), path)
            },
            Err(e) => panic!("Returned an unexpected error {e:?}"),
            Ok(_) => panic!("Did not return an error")
        }
    }

    fn build_cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs{
                reprocess_run
            })
        }
    }
}

#[cfg(test)]
mod process_message_execute_command_enumerate_urls_test {
    use assert_fs::fixture::TempDir;
    use crate::cli::SingleCli;
    use crate::data::{FulfillmentNode, Node};
    use crate::mountebank::*;
    use crate::persistence::{connect, find_run};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::Path;
    use support::{build_cli, build_config_file, build_config_location};
    use super::*;

    #[test]
    fn enumerates_links() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        setup_head_impostor(4560, true, Some("https://re.direct.one"));
        setup_head_impostor(4561, true, Some("https://re.direct.two"));

        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(&input, &cli, &config_file_location);

        tokio_test::block_on(execute_command(&config)).unwrap();

        let conn = connect(&db_path).unwrap();
        let run_1 = find_run(&conn, 1).unwrap();
        let run_2 = find_run(&conn, 2).unwrap();

        assert_eq!(
            run_1.data.parsed_mail.fulfillment_nodes,
            vec![
                FulfillmentNode {
                    hidden: Some(Node::new("https://re.direct.one")),
                    ..FulfillmentNode::new("http://localhost:4560")
                },
            ]
        );

        assert_eq!(
            run_2.data.parsed_mail.fulfillment_nodes,
            vec![
                FulfillmentNode {
                    hidden: Some(Node::new("https://re.direct.two")),
                    ..FulfillmentNode::new("http://localhost:4561")
                },
            ]
        );
    }

    fn build_config<'a>(
        message: &'a str,
        cli: &'a SingleCli,
        config_file_location: &'a Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            cli,
            config_file_location,
        ).unwrap()
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
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 1",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4560\">Click Me</a>",
        )
    }

    fn mail_body_2() -> String {
        format!(
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 2",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4561\">Click Me</a>",
        )
    }
}

#[cfg(test)]
mod process_message_execute_command_populate_from_rdap_tests {
    use assert_fs::fixture::TempDir;
    use chrono::prelude::*;
    use crate::cli::SingleCli;
    use crate::data::{
        Domain,
        DomainCategory,
        EmailAddressData,
        EmailAddresses,
        Registrar,
        ResolvedDomain
    };
    use crate::mountebank::{
        clear_all_impostors, setup_bootstrap_server, setup_dns_server, setup_ip_v4_server,
        DnsServerConfig, IpServerConfig,
    };
    use crate::persistence::{connect, find_run};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::Path;
    use support::{build_cli, build_config_file, build_config_location};
    use super::*;

    #[test]
    fn populates_rdap_data() {
        setup_mountebank();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(&input, &cli, &config_file_location);

        tokio_test::block_on(execute_command(&config)).unwrap();

        let conn = connect(&db_path).unwrap();
        let run_1 = find_run(&conn, 1).unwrap();
        let run_2 = find_run(&conn, 2).unwrap();

        assert_eq!(
            run_1.data.parsed_mail.email_addresses,
            EmailAddresses {
                from: vec![
                    EmailAddressData {
                        address: "scammer@fake.net".into(),
                        domain: Some(
                            Domain {
                                abuse_email_address: None,
                                category: DomainCategory::Other,
                                name: "fake.net".into(),
                                registration_date: Some(
                                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                                ),
                                resolved_domain: Some(
                                    ResolvedDomain {
                                        abuse_email_address: None,
                                        name: "fake.net".into(),
                                        registration_date: Some(
                                            Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()
                                        ),
                                    }
                                )
                            }
                        ),
                        registrar: Some(
                            Registrar {
                                name: Some("Reg One".into()),
                                abuse_email_address: Some("abuse@regone.zzz".into()),
                            }
                        ),
                    }
                ],
                links: vec![],
                reply_to: vec![],
                return_path: vec![]
            }
        );

        assert_eq!(
            run_2.data.parsed_mail.email_addresses,
            EmailAddresses {
                from: vec![
                    EmailAddressData {
                        address: "scammer@alsofake.net".into(),
                        domain: Some(
                            Domain {
                                abuse_email_address: None,
                                category: DomainCategory::Other,
                                name: "alsofake.net".into(),
                                registration_date: Some(
                                    Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                                ),
                                resolved_domain: Some(
                                    ResolvedDomain {
                                        abuse_email_address: None,
                                        name: "alsofake.net".into(),
                                        registration_date: Some(
                                            Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()
                                        ),
                                    }
                                )
                            }
                        ),
                        registrar: Some(
                            Registrar {
                                name: Some("Reg Six".into()),
                                abuse_email_address: Some("abuse@regsix.zzz".into()),
                            }
                        ),
                    }
                ],
                links: vec![],
                reply_to: vec![],
                return_path: vec![]
            }
        );
    }

    fn build_config<'a>(
        message: &'a str,
        cli: &'a SingleCli,
        config_file_location: &'a Path,
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            cli,
            config_file_location,
        ).unwrap()
    }

    fn setup_mountebank() {
        clear_all_impostors();
        setup_bootstrap_server();

        setup_dns_server(vec![
            DnsServerConfig {
                domain_name: "fake.net",
                handle: None,
                registrar: Some("Reg One"),
                abuse_email: Some("abuse@regone.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 12).unwrap()),
                response_code: 200,
            },
            DnsServerConfig {
                domain_name: "alsofake.net",
                handle: None,
                registrar: Some("Reg Six"),
                abuse_email: Some("abuse@regsix.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()),
                response_code: 200,
            },
        ]);

        setup_ip_v4_server(vec![IpServerConfig::response_200(
                "10.10.10.10",
                None,
                ("10.0.0.0", "10.255.255.255"),
                Some(&[("Acme Hosting", "registrant", "abuse@acmehost.zzz")]),
        )]);
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
            "{}\r\n{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "From: scammer@fake.net",
            "Subject: Dodgy Subject 1",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4545\">Click Me</a>",
        )
    }

    fn mail_body_2() -> String {
        format!(
            "{}\r\n{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "From: scammer@alsofake.net",
            "Subject: Dodgy Subject 2",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4546\">Click Me</a>",
        )
    }
}

#[cfg(test)]
mod process_message_execute_command_add_reportable_entities_tests {
    use assert_fs::fixture::TempDir;
    use crate::cli::SingleCli;
    use crate::data::{
        EmailAddresses,
        FulfillmentNode,
        FulfillmentNodesContainer,
        Node,
        ReportableEntities
    };
    use crate::mountebank::*;
    use crate::persistence::{connect, find_run};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::Path;
    use support::{build_cli, build_config_file, build_config_location};
    use super::*;

    #[test]
    fn adds_reportable_entities() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        setup_head_impostor(4560, true, Some("https://re.direct.one"));
        setup_head_impostor(4561, true, Some("https://re.direct.two"));

        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(&input, &cli, &config_file_location);

        tokio_test::block_on(execute_command(&config)).unwrap();

        let conn = connect(&db_path).unwrap();
        let run_1 = find_run(&conn, 1).unwrap();
        let run_2 = find_run(&conn, 2).unwrap();

        assert_eq!(
            run_1.data.reportable_entities,
            reportable_entities("http://localhost:4560", "https://re.direct.one")
        );

        assert_eq!(
            run_2.data.reportable_entities,
            reportable_entities("http://localhost:4561", "https://re.direct.two")
        );
    }

    fn build_config<'a>(
        message: &'a str,
        cli: &'a SingleCli,
        config_file_location: &'a Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            cli,
            config_file_location,
        ).unwrap()
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
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 1",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4560\">Click Me</a>",
        )
    }

    fn mail_body_2() -> String {
        format!(
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 2",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4561\">Click Me</a>",
        )
    }

    fn reportable_entities(visible_url: &str, hidden_url: &str) -> Option<ReportableEntities> {
        Some(
            ReportableEntities {
                delivery_nodes: vec![],
                email_addresses: EmailAddresses {
                    from: vec![],
                    links: vec![],
                    reply_to: vec![],
                    return_path: vec![]
                },
                fulfillment_nodes_container:
                    FulfillmentNodesContainer {
                        duplicates_removed: false,
                        nodes: vec![
                            FulfillmentNode {
                                hidden: Some(Node::new(hidden_url)),
                                investigable: true,
                                visible: Node::new(visible_url)
                            }
                        ],
                    }
            }
        )
    }
}

#[cfg(test)]
mod process_message_execute_command_add_notifications_tests {
    use assert_fs::fixture::TempDir;
    use chrono::*;
    use crate::cli::SingleCli;
    use crate::mailer::Entity;
    use crate::mountebank::*;
    use crate::notification::Notification;
    use crate::persistence::{connect, find_run};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::Path;
    use support::{build_cli, build_config_file, build_config_location};
    use super::*;

    #[test]
    fn adds_notifications_reportable_entities() {
        setup_mountebank();
        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");

        let cli = build_cli(None);

        let input = multiple_source_input();

        build_config_file(&config_file_location, &db_path);

        let config = build_config(&input, &cli, &config_file_location);

        tokio_test::block_on(execute_command(&config)).unwrap();

        let conn = connect(&db_path).unwrap();
        let run_1 = find_run(&conn, 1).unwrap();
        let run_2 = find_run(&conn, 2).unwrap();

        assert_eq!(
            run_1.data.notifications,
            notifications_for("http://re.directone.net", "abuse@regone.zzz")
        );

        assert_eq!(
            run_2.data.notifications,
            notifications_for("http://re.directtwo.net", "abuse@regsix.zzz")
        );
    }

    fn build_config<'a>(
        message: &'a str,
        cli: &'a SingleCli,
        config_file_location: &'a Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            cli,
            config_file_location,
        ).unwrap()
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
                abuse_email: Some("abuse@regsix.zzz"),
                registration_date: Some(Utc.with_ymd_and_hms(2022, 11, 18, 10, 11, 17).unwrap()),
                response_code: 200,
            },
        ]);
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
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 1",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4560\">Click Me</a>",
        )
    }

    fn mail_body_2() -> String {
        format!(
            "{}\r\n{}\r\n{}\r\n\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 2",
            "Content-Type: text/html",
            "<a href=\"http://localhost:4561\">Click Me</a>",
        )
    }

    fn notifications_for(entity: &str, email_address: &str) -> Vec<Notification> {
        vec![Notification::via_email(Entity::Node(entity.into()), String::from(email_address))]
    }
}

#[cfg(test)]
mod execute_command_invalid_config_tests {
    use support::build_broken_config;
    use super::*;

    #[test]
    fn returns_error_if_config_is_invalid() {
        let config = build_broken_config();

        let result = tokio_test::block_on(execute_command(&config));

        match result {
            Err(AppError::InvalidConfiguration(_)) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }
}

#[cfg(test)]
mod support {
    use assert_fs::TempDir;
    use crate::cli::{ProcessArgs, SingleCli, SingleCliCommands};
    use crate::service_configuration::{
        Configuration,
        FileConfig,
        ServiceConfiguration,
        ServiceType
    };
    use std::path::{Path, PathBuf};
    use sha2::{Digest, Sha256};

    pub fn sha256(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text);
        let sha = hasher.finalize();

        sha.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn build_config_location(temp: &TempDir) -> PathBuf {
        temp.path().join("phisher_eagle.conf")
    }

    pub fn build_config_file(config_file_location: &Path, db_path: &Path) {
        let contents = FileConfig {
            db_path: Some(db_path.to_str().unwrap().into()),
            rdap_bootstrap_host: Some("http://localhost:4545".into()),
            ..FileConfig::default()
        };

        confy::store_path(config_file_location, contents).unwrap();
    }

    pub fn build_config<'a>(
        message: Option<&'a str>,
        cli: &'a SingleCli,
        config_file_location: &'a Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            message,
            cli,
            config_file_location,
        ).unwrap()
    }

    pub fn build_cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run,
            })
        }
    }

    pub struct InvalidConfig;

    impl Configuration for InvalidConfig {
        fn abuse_notifications_author_name(&self) -> Option<&str> {
            None
        }

        fn abuse_notifications_from_address(&self) -> Option<&str> {
            None
        }

        fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
            vec![]
        }

        fn config_file_location(&self) -> &Path {
            Path::new("/does/not/matter")
        }

        fn db_path(&self) -> Option<&Path> {
            None
        }

        fn message_sources(&self) -> Option<&str> {
            None
        }

        fn rdap_bootstrap_host(&self) -> Option<&str> {
            None
        }

        fn reprocess_run_id(&self) -> Option<i64> {
            None
        }

        fn service_type(&self) -> &ServiceType {
            &ServiceType::ProcessMessage
        }

        fn store_config(&mut self) {
        }

        fn trusted_recipient(&self) -> Option<&str> {
            None
        }
    }

    pub fn build_broken_config() -> InvalidConfig {
        InvalidConfig {}
    }
}
