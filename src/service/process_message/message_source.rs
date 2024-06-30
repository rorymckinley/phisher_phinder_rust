use crate::errors::AppError;
use crate::message_source::MessageSource;
use crate::persistence::find_run;
use crate::result::AppResult;
use crate::service_configuration::Configuration;
use crate::sources::create_from_str;
use rusqlite::Connection;

pub fn build_message_sources<T: Configuration>(
    connection: &Connection,
    config: &T
) -> AppResult<Vec<MessageSource>> {
    if let Some(stdin_input) = config.message_sources() {
        let inputs = create_from_str(stdin_input);

        if !inputs.is_empty() {
            Ok(inputs)
        } else {
            Err(AppError::NoMessageSource)
        }
    } else if let Some(run_id) = config.reprocess_run_id() {
        if let Some(run) = find_run(connection, run_id) {
            Ok(vec![run.message_source])
        } else {
            Err(AppError::SpecifiedRunMissing)
        }
    } else {
        Err(AppError::NoMessageSource)
    }
}

#[cfg(test)]
mod build_message_sources_tests {
    use crate::authentication_results::AuthenticationResults;
    use crate::data::{
        EmailAddressData,
        EmailAddresses,
        FulfillmentNodesContainer,
        ParsedMail,
        OutputData,
        ReportableEntities
    };
    use crate::persistence::{persist_message_source, persist_run};
    use crate::run::Run;
    use crate::service_configuration::ServiceType;
    use std::path::Path;
    use super::*;

    #[test]
    fn builds_message_sources_from_stdin_if_present() {
        let conn = Connection::open_in_memory().unwrap();

        let config = TestConfig {
            message_sources: Some("Delivered-To: blah".into()),
            reprocess_run_id: None,
            service_type: ServiceType::ProcessMessage,
        };

        assert_eq!(
            vec![MessageSource::new("Delivered-To: blah")],
            build_message_sources(&conn, &config).unwrap()
        );
    }

    #[test]
    fn builds_inputs_from_run_id_if_no_stdin() {
        let conn = Connection::open_in_memory().unwrap();

        let _run_1 = build_run(&conn, 0);
        let run_2 = build_run(&conn, 1);
        let _run_3 = build_run(&conn, 2);

        let config = TestConfig {
            message_sources: None,
            reprocess_run_id: Some(run_2.id.into()),
            service_type: ServiceType::ProcessMessage,
        };

        assert_eq!(
            vec![MessageSource::persisted_record(run_2.message_source.id.unwrap(), "src 1")],
            build_message_sources(&conn, &config).unwrap()
        );
    }

    #[test]
    fn returns_an_error_if_no_stdin_or_run_id() {
        let conn = Connection::open_in_memory().unwrap();

        let config = TestConfig {
            message_sources: None,
            reprocess_run_id: None,
            service_type: ServiceType::ProcessMessage,
        };

       match build_message_sources(&conn, &config) {
            Err(AppError::NoMessageSource) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_an_error_if_run_id_cannot_be_found() {
        let conn = Connection::open_in_memory().unwrap();

        let _run_1 = build_run(&conn, 0);
        let _run_2 = build_run(&conn, 1);
        let _run_3 = build_run(&conn, 2);

        let config = TestConfig {
            message_sources: None,
            reprocess_run_id: Some(0),
            service_type: ServiceType::ProcessMessage,
        };

       match build_message_sources(&conn, &config) {
            Err(AppError::SpecifiedRunMissing) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_an_error_if_stdin_is_empty() {
        let conn = Connection::open_in_memory().unwrap();

        let _run_1 = build_run(&conn, 0);
        let _run_2 = build_run(&conn, 1);
        let _run_3 = build_run(&conn, 2);

        let config = TestConfig {
            message_sources: Some("".into()),
            reprocess_run_id: None,
            service_type: ServiceType::ProcessMessage,
        };

       match build_message_sources(&conn, &config) {
            Err(AppError::NoMessageSource) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    struct TestConfig {
        message_sources: Option<String>,
        reprocess_run_id: Option<i64>,
        service_type: ServiceType,
    }

    impl Configuration for TestConfig {
        fn abuse_notifications_author_name(&self) -> Option<&str>{
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
            self.message_sources.as_deref()
        }

        fn rdap_bootstrap_host(&self) -> Option<&str> {
            None
        }

        fn reprocess_run_id(&self) -> Option<i64> {
            self.reprocess_run_id
        }

        fn service_type(&self) -> &ServiceType {
            &self.service_type
        }

        fn store_config(&mut self) {
        }

        fn trusted_recipient(&self)-> Option<&str> {
            None
        }
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
