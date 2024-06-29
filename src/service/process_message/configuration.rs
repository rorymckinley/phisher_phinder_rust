use crate::errors::AppError;
use crate::service_configuration;
use std::path::Path;

pub fn extract_command_config<T>(config: &T) -> Result<Configuration, AppError>
where T: service_configuration::Configuration {
    check_for_source_data(config)?;

    if let Some(db_path) = config.db_path().as_ref() {
        Ok(Configuration {
            db_path,
            message_source: config.message_sources(),
            reprocess_run_id: config.reprocess_run_id(),
            trusted_recipient: config.trusted_recipient()
        })
    } else {
        Err(AppError::InvalidConfiguration("Please configure db_path.".into()))
    }
}

fn check_for_source_data<T>(config: &T) -> Result<(), AppError>
where T: service_configuration::Configuration {
    if config.reprocess_run_id().is_none() {
        match config.message_sources() {
            None => Err(AppError::NoMessageSource),
            Some(message_sources) => {
                if message_sources.is_empty() {
                    Err(AppError::NoMessageSource)
                } else {
                    Ok(())
                }
            }
        }
    } else {
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Configuration<'a> {
    pub db_path: &'a Path,
    pub message_source: Option<&'a str>,
    pub reprocess_run_id: Option<i64>,
    pub trusted_recipient: Option<&'a str>,
}

#[cfg(test)]
mod extract_command_config_tests {
    use crate::service_configuration::ServiceType;
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn returns_extracted_configuration_if_extract_command_config_for_run_from_stdin() {
        let config = merge(
            base_config(),
            OverrideConfig {
                message_sources: Some("message source".into()),
                reprocess_run_id: None,
                ..OverrideConfig::default()
            }
        );
        let extracted_config = Configuration {
            db_path: &PathBuf::from("/does/not/matter"),
            message_source: Some("message source"),
            reprocess_run_id: None,
            trusted_recipient: Some("mx.google.com"),
        };

        assert_eq!(extract_command_config(&config).unwrap(), extracted_config);
    }

    #[test]
    fn returns_extracted_config_if_extract_command_config_for_run_from_db() {
        let config = merge(
            base_config(),
            OverrideConfig {
                message_sources: None,
                reprocess_run_id: Some(999),
                ..OverrideConfig::default()
            }
        );

        let extracted_config = Configuration {
            db_path: &PathBuf::from("/does/not/matter"),
            message_source: None,
            reprocess_run_id: Some(999),
            trusted_recipient: Some("mx.google.com"),
        };

        assert_eq!(extract_command_config(&config).unwrap(), extracted_config);
    }

    #[test]
    fn returns_error_if_no_stdin_or_reprocess_run_id() {
        let config = base_config();
        let result = extract_command_config(&config);

        match result {
            Err(AppError::NoMessageSource) => (),
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_stdin_empty_string() {
        let config = merge(
            base_config(),
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
        let config = config_sans_db_path(base_config());

        let result = extract_command_config(&config);

        match result {
            Err(AppError::InvalidConfiguration(message)) => {
                assert_eq!(message, "Please configure db_path.");
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

    fn base_config() -> TestConfig {
        TestConfig {
            abuse_notifications_author_name: Some("Author Name".into()),
            abuse_notifications_from_address: Some("From Address".into()),
            config_file_location: PathBuf::from("/does/not/matter"),
            db_path: Some(PathBuf::from("/does/not/matter")),
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
