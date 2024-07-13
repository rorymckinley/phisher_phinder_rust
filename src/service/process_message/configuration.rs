use crate::errors::AppError;
use crate::message_source::MessageSource;
use crate::persistence::connect;
use crate::service::process_message::message_source::build_message_sources;
use crate::service_configuration;
use std::path::Path;

pub fn extract_command_config<T>(config: &T) -> Result<ConfigurationTwo, AppError>
where T: service_configuration::Configuration {
    if let Some(db_path) = config.db_path().as_ref() {
        if let Ok(db_connection) = connect(db_path) {
            let inputs = build_message_sources(&db_connection, config)?;

            Ok(ConfigurationTwo {
                db_connection,
                inputs,
                trusted_recipient: config.trusted_recipient()
            })
        } else {
            // TODO If we push the error conversion into connect, this code can be
            // collapsed into a ? call
            // NOTE There is a chance that this fails, due to invalid UTF-8
            // Not sure how likely it is to happen, but it is really hard to test,
            // so for now, allow it to panic
            Err(AppError::DatabasePathIncorrect(db_path.to_str().unwrap().into()))
        }
    } else {
        Err(AppError::InvalidConfiguration("Please configure db_path.".into()))
    }
}

#[derive(Debug, PartialEq)]
pub struct Configuration<'a> {
    pub db_path: &'a Path,
    pub message_source: Option<&'a str>,
    pub reprocess_run_id: Option<i64>,
    pub trusted_recipient: Option<&'a str>,
}

#[derive(Debug)]
pub struct ConfigurationTwo<'a> {
    pub db_connection: rusqlite::Connection,
    pub inputs: Vec<MessageSource>,
    pub trusted_recipient: Option<&'a str>,
}

#[cfg(test)]
mod extract_command_config_tests {
    use assert_fs::TempDir;
    use crate::service_configuration::ServiceType;
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn returns_extracted_configuration() {
        let temp = TempDir::new().unwrap();

        let config = merge(
            base_config(&temp),
            OverrideConfig {
                message_sources: Some("message source".into()),
                reprocess_run_id: None,
                ..OverrideConfig::default()
            }
        );

        let extracted_config = extract_command_config(&config).unwrap();

        assert_eq!(vec![MessageSource::new("message source")], extracted_config.inputs);
        assert_eq!(Some("mx.google.com"), extracted_config.trusted_recipient);
        assert_eq!(temp.join("db.sqlite3").to_str(), extracted_config.db_connection.path());
    }

    #[test]
    fn returns_error_if_no_stdin_or_reprocess_run_id() {
        let temp = TempDir::new().unwrap();

        let config = base_config(&temp);
        let result = extract_command_config(&config);

        match result {
            Err(AppError::NoMessageSource) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_stdin_empty_string() {
        let temp = TempDir::new().unwrap();

        let config = merge(
            base_config(&temp),
            OverrideConfig {
                message_sources: Some("".into()),
                reprocess_run_id: None,
                ..OverrideConfig::default()
            }
        );
        let result = extract_command_config(&config);

        match result {
            Err(AppError::NoMessageSource) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_db_path_not_set() {
        let temp = TempDir::new().unwrap();

        let config = config_sans_db_path(base_config(&temp));

        let result = extract_command_config(&config);

        match result {
            Err(AppError::InvalidConfiguration(message)) => {
                assert_eq!(message, "Please configure db_path.");
            },
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_db_connection_cannot_be_established() {
        let temp = TempDir::new().unwrap();

        let config = config_with_unreachable_db_path(base_config(&temp), &temp);

        let result = extract_command_config(&config);

        match result {
            Err(AppError::DatabasePathIncorrect(message)) => {
                let incorrect_path = temp.path().join("un/ob/tai/nium");
                let expected_message = incorrect_path.to_str().unwrap();
                assert_eq!(expected_message, message);
            },
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    fn merge(base: TestConfig, mods: OverrideConfig) -> TestConfig {
        let abuse_notifications_author_name = mods.abuse_notifications_author_name.or(
            base.abuse_notifications_author_name
        );
        let abuse_notifications_from_address = mods.abuse_notifications_from_address.or(
            base.abuse_notifications_from_address
        );
        let db_path = mods.db_path.or(base.db_path);
        let message_sources = mods.message_sources.or(base.message_sources);
        let rdap_bootstrap_host = mods.rdap_bootstrap_host.or(base.rdap_bootstrap_host);
        let reprocess_run_id = mods.reprocess_run_id.or(base.reprocess_run_id);
        let trusted_recipient = mods.trusted_recipient.or(base.trusted_recipient);

        TestConfig {
            abuse_notifications_author_name,
            abuse_notifications_from_address,
            db_path,
            message_sources,
            rdap_bootstrap_host,
            reprocess_run_id,
            trusted_recipient,
            ..base
        }
    }

    fn base_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            abuse_notifications_author_name: Some("Author Name".into()),
            abuse_notifications_from_address: Some("From Address".into()),
            config_file_location: PathBuf::from("/does/not/matter"),
            db_path: Some(temp.path().join("db.sqlite3")),
            message_sources: None,
            rdap_bootstrap_host: Some("http://localhost:4545".into()),
            reprocess_run_id: None,
            service_type: ServiceType::ProcessMessage,
            trusted_recipient: Some("mx.google.com".into())
        }
    }

    fn config_sans_db_path(base_config: TestConfig) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            db_path: None,
            ..base_config
        }
    }

    fn config_with_unreachable_db_path(base_config: TestConfig, temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            db_path: Some(temp.path().join("un/ob/tai/nium")),
            ..base_config
        }
    }
    #[derive(Default)]
    struct OverrideConfig {
        abuse_notifications_author_name: Option<String>,
        abuse_notifications_from_address: Option<String>,
        db_path: Option<PathBuf>,
        message_sources: Option<String>,
        rdap_bootstrap_host: Option<String>,
        reprocess_run_id: Option<i64>,
        trusted_recipient: Option<String>,
    }

    struct TestConfig {
        abuse_notifications_author_name: Option<String>,
        abuse_notifications_from_address: Option<String>,
        config_file_location: PathBuf,
        db_path: Option<PathBuf>,
        message_sources: Option<String>,
        rdap_bootstrap_host: Option<String>,
        reprocess_run_id: Option<i64>,
        service_type: ServiceType,
        trusted_recipient: Option<String>,
    }

    impl service_configuration::Configuration for TestConfig {
        fn abuse_notifications_author_name(&self) -> Option<&str>{
            self.abuse_notifications_author_name.as_deref()
        }

        fn abuse_notifications_from_address(&self) -> Option<&str> {
            self.abuse_notifications_from_address.as_deref()
        }

        fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
            vec![]
        }

        fn config_file_location(&self) -> &Path {
            &self.config_file_location
        }

        fn db_path(&self) -> Option<&Path> {
            self.db_path.as_deref()
        }

        fn message_sources(&self) -> Option<&str> {
            self.message_sources.as_deref()
        }

        fn rdap_bootstrap_host(&self) -> Option<&str> {
            self.rdap_bootstrap_host.as_deref()
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
            self.trusted_recipient.as_deref()
        }
    }
}
