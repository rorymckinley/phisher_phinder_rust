use crate::errors::AppError;
use crate::message_source::MessageSource;
use crate::persistence::connect;
use crate::service::process_message::message_source::build_message_sources;
use crate::service_configuration;

pub fn extract_command_config<T>(config: &T) -> Result<Configuration, AppError>
where T: service_configuration::Configuration {
    if let Some(db_path) = config.db_path().as_ref() {
        if let Ok(db_connection) = connect(db_path) {
            let inputs = build_message_sources(&db_connection, config)?;

            // TODO for now we assume we will always have author name and from address
            let abuse_notifications = AbuseNotificationConfiguration {
                author_name: config.abuse_notifications_author_name().unwrap().into(),
                from_address: config.abuse_notifications_from_address().unwrap().into(),
                test_recipient: config.test_recipient().map(|s| s.into()),
            };

            let email_notifications = if config.send_abuse_notifications() {
                match (config.smtp_host_uri(), config.smtp_password(), config.smtp_username()) {
                    (Some(host_uri), Some(password), Some(username)) => {
                        Some(EmailNotificationConfiguration {
                            host_uri: host_uri.into(),
                            password: password.into(),
                            username: username.into()
                        })
                    },
                    _ => {
                        return Err(AppError::InvalidConfiguration(
                            "Please configure smtp_host_uri, smtp_password and smtp_username \
                             if you wish to send notifications".into()
                        ));
                    }
                }
            } else {
                None
            };

            Ok(Configuration {
                abuse_notifications: Some(abuse_notifications),
                db_connection,
                email_notifications,
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

#[derive(Debug)]
pub struct Configuration<'a> {
    pub abuse_notifications: Option<AbuseNotificationConfiguration>,
    pub db_connection: rusqlite::Connection,
    pub email_notifications: Option<EmailNotificationConfiguration>,
    pub inputs: Vec<MessageSource>,
    pub trusted_recipient: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct AbuseNotificationConfiguration {
    pub author_name: String,
    pub from_address: String,
    pub test_recipient: Option<String>
}

#[derive(Debug, PartialEq)]
pub struct EmailNotificationConfiguration {
    pub host_uri: String,
    pub password: String,
    pub username: String
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

        let expected_abuse_notifications = AbuseNotificationConfiguration {
            author_name: "Author Name".into(),
            from_address: "From Address".into(),
            test_recipient: None,
        };

        assert_eq!(vec![MessageSource::new("message source")], extracted_config.inputs);
        assert_eq!(Some("mx.google.com"), extracted_config.trusted_recipient);
        assert_eq!(temp.join("db.sqlite3").to_str(), extracted_config.db_connection.path());
        assert_eq!(Some(expected_abuse_notifications), extracted_config.abuse_notifications);
        assert_eq!(None, extracted_config.email_notifications);
    }

    #[test]
    fn sets_email_notifications_configuration_if_send_notifications_enabled() {
        let temp = TempDir::new().unwrap();

        let config = send_notifications_config(&temp);

        let extracted_config = extract_command_config(&config).unwrap();

        let expected_abuse_notifications = AbuseNotificationConfiguration {
            author_name: "Author Name".into(),
            from_address: "From Address".into(),
            test_recipient: None,
        };

        let expected_email_notifications = EmailNotificationConfiguration {
            host_uri: "smtp.host.zzz".into(),
            password: "password".into(),
            username: "username".into()
        };

        assert_eq!(vec![MessageSource::new("message source")], extracted_config.inputs);
        assert_eq!(Some("mx.google.com"), extracted_config.trusted_recipient);
        assert_eq!(temp.join("db.sqlite3").to_str(), extracted_config.db_connection.path());
        assert_eq!(Some(expected_abuse_notifications), extracted_config.abuse_notifications);
        assert_eq!(Some(expected_email_notifications), extracted_config.email_notifications);
    }

    #[test]
    fn sets_test_recipient_if_required() {
        let temp = TempDir::new().unwrap();

        let config = test_recipient_config(&temp);

        let extracted_config = extract_command_config(&config).unwrap();

        let expected_abuse_notifications = AbuseNotificationConfiguration {
            author_name: "Author Name".into(),
            from_address: "From Address".into(),
            test_recipient: Some("recipient@test.zzz".into()),
        };

        assert_eq!(Some(expected_abuse_notifications), extracted_config.abuse_notifications);
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

    #[test]
    fn returns_error_if_send_notifications_but_no_smtp_host_uri() {
        let temp = TempDir::new().unwrap();

        let config = send_notifications_no_smtp_host_uri_config(&temp);

        let result = extract_command_config(&config);

        match result {
            Err(AppError::InvalidConfiguration(message)) => {
                let expected = "Please configure smtp_host_uri, smtp_password and smtp_username \
                                if you wish to send notifications";
                assert_eq!(message, expected);
            },
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_send_notifications_but_no_smtp_password() {
        let temp = TempDir::new().unwrap();

        let config = send_notifications_no_smtp_password_config(&temp);

        let result = extract_command_config(&config);

        match result {
            Err(AppError::InvalidConfiguration(message)) => {
                let expected = "Please configure smtp_host_uri, smtp_password and smtp_username \
                                if you wish to send notifications";
                assert_eq!(message, expected);
            },
            Err(e) => panic!("Returned unexpected {e}"),
            Ok(_) => panic!("Did not return error")
        }
    }

    #[test]
    fn returns_error_if_send_notifications_but_no_smtp_username() {
        let temp = TempDir::new().unwrap();

        let config = send_notifications_no_smtp_username_config(&temp);

        let result = extract_command_config(&config);

        match result {
            Err(AppError::InvalidConfiguration(message)) => {
                let expected = "Please configure smtp_host_uri, smtp_password and smtp_username \
                                if you wish to send notifications";
                assert_eq!(message, expected);
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
            send_notifications: false,
            service_type: ServiceType::ProcessMessage,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            test_recipient: None,
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

    fn send_notifications_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            smtp_host_uri: Some("smtp.host.zzz".into()),
            smtp_password: Some("password".into()),
            smtp_username: Some("username".into()),
            send_notifications: true,
            ..base_config(temp)
        }
    }

    fn send_notifications_no_smtp_host_uri_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            smtp_host_uri: None,
            smtp_password: Some("password".into()),
            smtp_username: Some("username".into()),
            send_notifications: true,
            ..base_config(temp)
        }
    }

    fn send_notifications_no_smtp_password_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            smtp_host_uri: Some("smtp.host.zzz".into()),
            smtp_password: None,
            smtp_username: Some("username".into()),
            send_notifications: true,
            ..base_config(temp)
        }
    }
    fn send_notifications_no_smtp_username_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            smtp_host_uri: Some("smtp.host.zzz".into()),
            smtp_password: Some("password".into()),
            smtp_username: None,
            send_notifications: true,
            ..base_config(temp)
        }
    }

    fn test_recipient_config(temp: &TempDir) -> TestConfig {
        TestConfig {
            message_sources: Some("message source".into()),
            test_recipient: Some("recipient@test.zzz".into()),
            ..base_config(temp)
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
        send_notifications: bool,
        service_type: ServiceType,
        smtp_host_uri: Option<String>,
        smtp_password: Option<String>,
        smtp_username: Option<String>,
        test_recipient: Option<String>,
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

        fn send_abuse_notifications(&self) -> bool {
            self.send_notifications
        }

        fn service_type(&self) -> &ServiceType {
            &self.service_type
        }

        fn smtp_host_uri(&self) -> Option<&str> {
            self.smtp_host_uri.as_deref()
        }

        fn smtp_password(&self) -> Option<&str> {
            self.smtp_password.as_deref()
        }

        fn smtp_username(&self) -> Option<&str> {
            self.smtp_username.as_deref()
        }

        fn store_config(&mut self) {
        }

        fn test_recipient(&self) -> Option<&str> {
            self.test_recipient.as_deref()
        }

        fn trusted_recipient(&self)-> Option<&str> {
            self.trusted_recipient.as_deref()
        }
    }
}
