use crate::analyser::Analyser;
use crate::cli::SingleCli;
use crate::data::OutputData;
use crate::enumerator::enumerate;
use crate::errors::AppError;
use crate::persistence::{connect, find_run, persist_message_source, persist_run};
use crate::populator::populate;
use crate::reporter::add_reportable_entities;
use crate::run::Run;
use crate::run_presenter::present;
use crate::sources::create_from_str;
use mail_parser::*;
use rusqlite::Connection;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use test_friendly_rdap_client::Client;
use thiserror::Error;
use tokio::task::JoinError;

pub trait Configuration {
    fn db_path(&self) -> &PathBuf;

    fn message_sources(&self) -> Option<&str>;

    fn rdap_bootstrap_host(&self) -> Option<&str>;

    fn reprocess_run_id(&self) -> Option<i64>;

    fn trusted_recipient(&self)-> &str;
}

#[derive(Debug, Error, PartialEq)]
pub enum ServiceConfigurationError {
    #[error("{0} is a required ENV variable")]
    MissingEnvVar(String),
    #[error("Please pass message source to STDIN or reprocess a run.")]
    NoMessageSource,
}

#[derive(Debug, PartialEq)]
pub struct ServiceConfiguration<'a> {
    db_path: PathBuf,
    message_source: Option<&'a str>,
    rdap_bootstrap_host: Option<String>,
    reprocess_run_id: Option<i64>,
    trusted_recipient: String,
}

impl<'a> ServiceConfiguration<'a> {
    pub fn new<I>(
        message_source: Option<&'a str>,
        cli_parameters: &SingleCli,
        env_vars_iterator: I
    ) -> Result<Self, ServiceConfigurationError>
    where I: Iterator<Item = (String, String)>
    {
        if message_source.is_none() && cli_parameters.reprocess_run.is_none() {
            return Err(ServiceConfigurationError::NoMessageSource);
        }

        let env_vars: HashMap<String, String> = env_vars_iterator.collect();

        Ok(Self {
            db_path: Path::new(&Self::extract_required_env_var(&env_vars, "PP_DB_PATH")?).to_path_buf(),
            message_source,
            rdap_bootstrap_host: Self::extract_optional_env_var(&env_vars, "RDAP_BOOTSTRAP_HOST"),
            reprocess_run_id: cli_parameters.reprocess_run,
            trusted_recipient: Self::extract_required_env_var(&env_vars, "PP_TRUSTED_RECIPIENT")?,
        })
    }

    fn extract_required_env_var(
        vars: &HashMap<String, String>,
        var_name: &str,
    ) -> Result<String, ServiceConfigurationError> {
        if let Some(val_ref) = vars.get(var_name) {
            if !val_ref.is_empty() {
                Ok(val_ref.to_string())
            } else {
                Err(ServiceConfigurationError::MissingEnvVar(var_name.into()))
            }
        } else {
            Err(ServiceConfigurationError::MissingEnvVar(var_name.into()))
        }
    }

    fn extract_optional_env_var(
        vars: &HashMap<String, String>,
        var_name: &str,
    ) -> Option<String> {
        if let Some(val) = vars.get(var_name) {
            if !val.is_empty() {
                Some(val.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> Configuration for ServiceConfiguration<'a> {
    fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    fn message_sources(&self) -> Option<&'a str> {
        self.message_source
    }

    fn rdap_bootstrap_host(&self) -> Option<&str> {
        self.rdap_bootstrap_host.as_deref()
    }

    fn reprocess_run_id(&self) -> Option<i64> {
        self.reprocess_run_id
    }

    fn trusted_recipient(&self) -> &str {
        &self.trusted_recipient
    }
}

#[cfg(test)]
mod service_configuration_tests {
    use support::{cli, env_var_iterator};
    use super::*;

    #[test]
    fn initialises_an_instance() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        ).unwrap();

        let expected = ServiceConfiguration {
            db_path: "/path/to/db".into(),
            message_source: Some("message source"),
            rdap_bootstrap_host: None,
            reprocess_run_id: Some(99),
            trusted_recipient: "foo.com".into()
        };

        assert_eq!(expected, config);
    }

    #[test]
    fn returns_err_if_no_db_path_env_var() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(None, Some("foo.com"), None)
        );

        match result {
            Err(ServiceConfigurationError::MissingEnvVar(e)) => {
                assert_eq!("PP_DB_PATH", e)
            },
            Err(_) => panic!("Unexpected error response"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_err_if_db_path_env_var_empty_string() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(Some(""), Some("foo.com"), None)
        );

        match result {
            Err(ServiceConfigurationError::MissingEnvVar(e)) => {
                assert_eq!("PP_DB_PATH", e)
            },
            Err(_) => panic!("Unexpected error response"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_err_if_no_trusted_recipient_env_var() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(Some("/path/to/db"), None, None)
        );

        match result {
            Err(ServiceConfigurationError::MissingEnvVar(e)) => {
                assert_eq!("PP_TRUSTED_RECIPIENT", e)
            },
            Err(_) => panic!("Unexpected error response"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_err_if_trusted_recipient_env_var_empty_string() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(Some("/path/to/db"), Some(""), None)
        );

        match result {
            Err(ServiceConfigurationError::MissingEnvVar(e)) => {
                assert_eq!("PP_TRUSTED_RECIPIENT", e)
            },
            Err(_) => panic!("Unexpected error response"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_err_if_no_message_source_or_reprocess_run() {
        let result = ServiceConfiguration::new(
            None,
            &cli(None),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        );

        match result {
            Err(e) => assert_eq!(ServiceConfigurationError::NoMessageSource, e),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_config_if_no_message_source_but_reprocess_run_id() {
        let result = ServiceConfiguration::new(
            None,
            &cli(Some(99)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        );

        assert!(result.is_ok());
    }

    #[test]
    fn returns_config_if_message_source_but_no_reprocess_run_id() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        );

        assert!(result.is_ok());
    }

    #[test]
    fn returns_db_path() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        ).unwrap();

        assert_eq!(&Path::new("/path/to/db").to_path_buf(), config.db_path());
    }

    #[test]
    fn returns_message_sources() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        ).unwrap();

        assert_eq!(Some("message source"), config.message_sources());
    }

    #[test]
    fn returns_reprocess_run_id() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        ).unwrap();

        assert_eq!(Some(999), config.reprocess_run_id());
    }

    #[test]
    fn returns_trusted_recipient() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), None)
        ).unwrap();

        assert_eq!("foo.com", config.trusted_recipient());
    }

    #[test]
    fn returns_rdap_bootstrap_host() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), Some("localhost:4545"))
        ).unwrap();

        assert_eq!(Some("localhost:4545"), config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_none_if_rdap_host_empty_string() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(Some("/path/to/db"), Some("foo.com"), Some(""))
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }
}

pub struct Service {
}

impl Service {
    pub async fn process_message<T>(config: &T) -> Result<String, AppError>
    where T: Configuration {
        let connection = Self::setup_connection(config)?;

        let input_data = Self::process_message_source(&connection, config)?;

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

        let records_with_reportable_entities = Self::add_reportable_entities(populate_tasks).await;

        let run_result = Self::persist_runs(&connection, records_with_reportable_entities)?;

        // TODO The error in the Result is a tuple of (Connection, Error)
        // Add error conversion for this
        connection.close().unwrap();

        match run_result {
            RunStorageResult::MultipleRuns(count) => Ok(format!("{count} messages processed.")),
            RunStorageResult::SingleRun(boxed_run) => {
                present(*boxed_run)
            }
        }
    }

    fn process_message_source<T>(connection: &Connection,  config: &T) -> Result<Vec<OutputData>, AppError>
    where T: Configuration {

        if let Some(message_sources)= config.message_sources() {
            let outputs = create_from_str(message_sources)
                .into_iter()
                .map(|message_source| {
                    let persisted_source = persist_message_source(&connection, message_source);

                    // TODO Better error handling
                    let parsed_mail = Message::parse(persisted_source.data.as_bytes()).unwrap();

                    let analyser = Analyser::new(&parsed_mail);

                    // TODO Better error handling
                    let parsed_mail = analyser.analyse(config).unwrap();

                    //TODO rework analyser.delivery_nodes to take service configuration
                    OutputData::new(parsed_mail, persisted_source)
                })
                .collect::<Vec<_>>();

            Ok(outputs)
        } else if let Some(run_id) = config.reprocess_run_id() {
            let connection = Self::setup_connection(config)?;

            if let Some(run) = find_run(&connection, run_id) {
                // TODO Better error handling
                let parsed_mail = Message::parse(run.message_source.data.as_bytes()).unwrap();

                let analyser = Analyser::new(&parsed_mail);

                // TODO Better error handling
                let parsed_mail = analyser.analyse(config).unwrap();

                //TODO rework analyser.delivery_nodes to take service configuration
                let output = OutputData::new(parsed_mail, run.message_source);

                // TODO The error in the Result is a tuple of (Connection, Error)
                // Add error conversion for this
                connection.close().unwrap();

                Ok(vec![output])
            } else {
                // TODO The error in the Result is a tuple of (Connection, Error)
                // Add error conversion for this
                connection.close().unwrap();

                Err(AppError::SpecifiedRunMissing)
            }
        } else {
            Err(AppError::NothingToProcess)
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

    async fn add_reportable_entities(
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

    fn setup_connection<T>(config: &T) -> Result<Connection, AppError>
    where T: Configuration {
        connect(config.db_path()).map_err(|e| {
            println!("{e:?}");
            // NOTE There is a chance that this fails, due to invalid UTF-8
            // Not sure how likely it is to happen, but it is really hard to test,
            // so for now, allow it to panic
            let path = config.db_path().to_str().unwrap();
            AppError::DatabasePathIncorrect(path.into())
        })
    }
}

enum RunStorageResult {
    MultipleRuns(usize),
    SingleRun(Box<Run>),
}

#[cfg(test)]
mod service_process_message_from_stdin_tests {
    use assert_fs::fixture::TempDir;
    use crate::message_source::MessageSource;
    use crate::mountebank::{clear_all_impostors, setup_bootstrap_server};
    use crate::persistence::{connect, find_runs_for_message_source, get_record};
    use crate::run::Run;
    use support::{build_config, sha256};

    use super::*;

    #[test]
    fn creates_messages_sources_provided_via_input() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        let config = build_config(Some(&input), None, &db_path);

        let result = tokio_test::block_on(Service::process_message(&config));

        assert!(result.is_ok());

        let persisted_hashes = lookup_message_source_hashes(&db_path);

        assert!(persisted_hashes.contains(&sha256(&mail_body_1())));
        assert!(persisted_hashes.contains(&sha256(&mail_body_2())));
    }

    #[test]
    fn persists_a_run_for_each_message_source() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        let config = build_config(Some(&input), None, &db_path);

        let _ = tokio_test::block_on(Service::process_message(&config));

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

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");
        let input = multiple_source_input();

        let config = build_config(Some(&input), None, &db_path);

        let output = tokio_test::block_on(Service::process_message(&config)).unwrap();

        assert_eq!(String::from("2 messages processed."), output);
    }

    #[test]
    fn returns_the_run_details_if_single_message() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");
        let input = single_source_input();

        let config = build_config(Some(&input), None, &db_path);

        let output = tokio_test::block_on(Service::process_message(&config)).unwrap();

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
mod service_process_message_rerun_tests {
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
    use crate::persistence::{connect, find_runs_for_message_source};
    use rusqlite::Connection;
    use support::build_config;

    use super::*;

    #[test]
    fn reruns_an_existing_run() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let _run_1_id = build_run(&conn, 0);
        let run_2_id = build_run(&conn, 1);
        let _run_3_id = build_run(&conn, 2);

        let config = build_config(None, Some(run_2_id), &db_path);

        let result = tokio_test::block_on(Service::process_message(&config));

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
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let _run_1_id = build_run(&conn, 0);
        let run_2_id = build_run(&conn, 1);
        let run_3_id = build_run(&conn, 2);

        let config = build_config(None, Some(run_2_id), &db_path);

        let _ = tokio_test::block_on(Service::process_message(&config));

        let run_2 = find_run(&conn, run_2_id).unwrap();

        let run_2_rerun = find_run(&conn, run_3_id + 1).unwrap();

        assert_eq!(run_2.data.message_source, run_2_rerun.data.message_source);
    }

    #[test]
    fn raises_error_if_run_does_not_exist() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        let conn = connect(&db_path).unwrap();

        let run_id = build_run(&conn, 0);

        let config = build_config(None, Some(run_id + 100), &db_path);

        match tokio_test::block_on(Service::process_message(&config)) {
            Err(AppError::SpecifiedRunMissing) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
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
mod service_process_message_common_errors_tests {
    use assert_fs::fixture::TempDir;

    use super::*;

    #[test]
    fn returns_err_if_db_cannot_be_initialised() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("un/ob/tai/nium");

        let config = ServiceConfiguration::new(
            Some("message_source"),
            &cli(None),
            env_var_iterator(Some(db_path.to_str().unwrap()), Some("foo.com"))
        ).unwrap();

        let result = tokio_test::block_on(Service::process_message(&config));

        match result {
            Err(AppError::DatabasePathIncorrect(path)) => {
                assert_eq!(db_path.to_str().unwrap(), path)
            },
            Err(e) => panic!("Returned an unexpected error {e:?}"),
            Ok(_) => panic!("Did not return an error")
        }
    }

    #[test]
    fn returns_error_if_no_input_or_rerun_id() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        let config = EmptyInputConfiguration { db_path, message_source: None };

        match tokio_test::block_on(Service::process_message(&config)) {
            Err(AppError::NothingToProcess) => (),
            Err(e) => panic!("Received unexpected error {e}"),
            Ok(_) => panic!("Did not receive an error")
        }
    }

    fn env_var_iterator(
        db_path_option: Option<&str>,
        trusted_recipient_option: Option<&str>
    ) -> Box<dyn Iterator<Item = (String, String)>>
    {
        let mut v: Vec<(String, String)> = vec![];

        if let Some(db_path) = db_path_option {
            v.push(("PP_DB_PATH".into(), db_path.into()));
        }

        if let Some(trusted_recipient) = trusted_recipient_option {
            v.push(("PP_TRUSTED_RECIPIENT".into(), trusted_recipient.into()))
        }

        Box::new(v.into_iter())
    }

    fn cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli { reprocess_run }
    }

    struct EmptyInputConfiguration<'a> { db_path: PathBuf, message_source: Option<&'a str> }

    impl<'a> Configuration for EmptyInputConfiguration<'a> {
        fn db_path(&self) -> &PathBuf {
            &self.db_path
        }

        fn message_sources(&self) -> Option<&'a str> {
            self.message_source
        }

        fn rdap_bootstrap_host(&self) -> Option<&'a str> {
            None
        }

        fn reprocess_run_id(&self) -> Option<i64> {
            None
        }

        fn trusted_recipient(&self) -> &str {
            ""
        }
    }
}

#[cfg(test)]
mod service_process_message_enumerate_urls_test {
    use assert_fs::fixture::TempDir;
    use crate::data::{FulfillmentNode, Node};
    use crate::mountebank::*;
    use crate::persistence::connect;
    use support::{cli, env_var_iterator};
    use super::*;

    #[test]
    fn enumerates_links() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        setup_head_impostor(4560, true, Some("https://re.direct.one"));
        setup_head_impostor(4561, true, Some("https://re.direct.two"));

        let input = multiple_source_input();

        let config = build_config(&input, &db_path);

        tokio_test::block_on(Service::process_message(&config)).unwrap();

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

    fn build_config<'a>(message: &'a str, db_path: &Path) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            &cli(None),
            env_var_iterator(
                Some(db_path.to_str().unwrap()),
                Some("foo.com"),
                Some("http://localhost:4545")
            )
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
mod service_process_message_populate_from_rdap_tests {
    use assert_fs::fixture::TempDir;
    use chrono::prelude::*;
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
    use support::{cli, env_var_iterator};
    use super::*;

    #[test]
    fn populates_rdap_data() {
        setup_mountebank();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        let input = multiple_source_input();

        let config = build_config(&input, &db_path);

        tokio_test::block_on(Service::process_message(&config)).unwrap();

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

    fn build_config<'a>(message: &'a str, db_path: &Path) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            &cli(None),
            env_var_iterator(
                Some(db_path.to_str().unwrap()),
                Some("foo.com"),
                Some("http://localhost:4545")
            )
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
mod service_process_message_add_reportable_entities_tests {
    use assert_fs::fixture::TempDir;
    use crate::data::{
        EmailAddresses,
        FulfillmentNode,
        FulfillmentNodesContainer,
        Node,
        ReportableEntities
    };
    use crate::mountebank::*;
    use crate::persistence::connect;
    use support::{cli, env_var_iterator};
    use super::*;

    #[test]
    fn adds_reportable_entities() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");

        setup_head_impostor(4560, true, Some("https://re.direct.one"));
        setup_head_impostor(4561, true, Some("https://re.direct.two"));

        let input = multiple_source_input();

        let config = build_config(&input, &db_path);

        tokio_test::block_on(Service::process_message(&config)).unwrap();

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

    fn build_config<'a>(message: &'a str, db_path: &Path) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            Some(message),
            &cli(None),
            env_var_iterator(
                Some(db_path.to_str().unwrap()),
                Some("foo.com"),
                Some("http://localhost:4545")
            )
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
                                visible: Node::new(visible_url)
                            }
                        ],
                    }
            }
        )
    }
}

#[cfg(test)]
mod support {
    use sha2::{Digest, Sha256};

    use super::*;

    pub fn sha256(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text);
        let sha = hasher.finalize();

        sha.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn build_config<'a>(
        message: Option<&'a str>,
        reprocess_run: Option<i64>,
        db_path: &Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            message,
            &cli(reprocess_run),
            env_var_iterator(
                Some(db_path.to_str().unwrap()),
                Some("foo.com"),
                Some("http://localhost:4545"))
        ).unwrap()
    }

    pub fn env_var_iterator(
        db_path_option: Option<&str>,
        trusted_recipient_option: Option<&str>,
        rdap_bootstrap_host_option: Option<&str>
    ) -> Box<dyn Iterator<Item = (String, String)>>
    {
        let mut v: Vec<(String, String)> = vec![];

        if let Some(db_path) = db_path_option {
            v.push(("PP_DB_PATH".into(), db_path.into()));
        }

        if let Some(trusted_recipient) = trusted_recipient_option {
            v.push(("PP_TRUSTED_RECIPIENT".into(), trusted_recipient.into()))
        }

        if let Some(rdap_bootstrap_host) = rdap_bootstrap_host_option {
            v.push(("RDAP_BOOTSTRAP_HOST".into(), rdap_bootstrap_host.into()))
        }

        Box::new(v.into_iter())
    }

    pub fn cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli { reprocess_run }
    }
}
