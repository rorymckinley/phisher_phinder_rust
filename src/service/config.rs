use crate::errors::AppError;
use crate::service_configuration::{ConfigServiceCommands, Configuration, ServiceType};

pub fn execute_command<T>(config: &T) -> Result<String, AppError>
    where T: Configuration {
    match config.service_type() {
        ServiceType::Config(ConfigServiceCommands::Location) => {
            if let Some(location) = config.config_file_location().to_str() {
                Ok(location.into())
            } else {
                Err(AppError::ConfigFileLocation)
            }
        },
        ServiceType::Config(ConfigServiceCommands::Show) => {
            Ok(present_config_file_entries(config.config_file_entries()))
        },
        ServiceType::Config(ConfigServiceCommands::Set) => {
            config.store_config();

            Ok(present_config_file_entries(config.config_file_entries()))
        }
        _ => {
            Err(AppError::Irrecoverable)
        }
    }
}

fn present_config_file_entries(mut entries: Vec<(String, Option<String>)>) -> String {
    entries.sort_by(|(key_a, _val_a), (key_b, _val_b)| key_a.partial_cmp(key_b).unwrap());

    entries
        .iter()
        .fold(String::from(""), |acc, (key, val_option)| {
            let empty_value = String::from("");
            let val = val_option.as_ref().unwrap_or(&empty_value);

            format!("{acc}{key}: {val}\n")
        })
}

#[cfg(test)]
mod execute_command_show_config_location_tests {
    use crate::service_configuration::ServiceType;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn returns_config_path_from_configuration() {
        let config = TestConfiguration { path: PathBuf::from("/path/to/config") };

        assert_eq!(String::from("/path/to/config"), execute_command(&config).unwrap());
    }

    #[test]
    fn returns_error_if_config_path_cannot_be_converted_to_string() {
        let config = TestConfiguration { path: broken_path() };

        match execute_command(&config) {
            Ok(_) => panic!("Unexpected OK"),
            Err(AppError::ConfigFileLocation) => (),
            Err(_) => panic!("Unexpected error")
        }
    }

    struct TestConfiguration { path: PathBuf }

    impl Configuration for TestConfiguration {
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
            &self.path
        }

        fn db_path(&self) -> Option<&PathBuf> {
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
            &ServiceType::Config(ConfigServiceCommands::Location)
        }

        fn store_config(&self) {
        }

        fn trusted_recipient(&self)-> Option<&str> {
            None
        }
    }

    fn broken_path() -> PathBuf {
        let invalid_utf8_bytes = vec![0xFF, 0xFF];

        let os_string = OsString::from_vec(invalid_utf8_bytes);

        PathBuf::from(os_string)
    }
}

#[cfg(test)]
mod execute_command_show_config_tests {
    use crate::service_configuration::ServiceType;
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn returns_the_contents_of_the_config() {
        let config = config_with_entries();

        let expected: String  = "\
                                 setting_a: value_99\n\
                                 setting_b: value_101\n\
                                 setting_c: value_201\n\
                                 ".into();

        assert_eq!(expected, execute_command(&config).unwrap());
    }

    #[test]
    fn returns_config_with_empty_entries() {
        let config = config_with_empty_entries();

        let expected: String  = "\
                                 setting_a: \n\
                                 setting_b: \n\
                                 setting_c: \n\
                                 ".into();

        assert_eq!(expected, execute_command(&config).unwrap());
    }

    struct TestConfiguration { entries: Vec<(String, Option<String>)> }

    impl Configuration for TestConfiguration {
        fn abuse_notifications_author_name(&self) -> Option<&str> {
            None
        }

        fn abuse_notifications_from_address(&self) -> Option<&str> {
            None
        }

        fn config_file_location(&self) -> &Path {
            Path::new("/does/not/matter")
        }

        fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
            self.entries.clone()
        }

        fn db_path(&self) -> Option<&PathBuf> {
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
            &ServiceType::Config(ConfigServiceCommands::Show)
        }

        fn store_config(&self) {
        }

        fn trusted_recipient(&self)-> Option<&str> {
            None
        }
    }

    fn config_with_entries() -> TestConfiguration {
        let entries = vec![
            ("setting_b".into(), Some("value_101".into())),
            ("setting_a".into(), Some("value_99".into())),
            ("setting_c".into(), Some("value_201".into())),
        ];

        TestConfiguration { entries }
    }

    fn config_with_empty_entries() -> TestConfiguration {
        let entries = vec![
            ("setting_b".into(), None),
            ("setting_a".into(), None),
            ("setting_c".into(), None),
        ];

        TestConfiguration { entries }
    }
}

#[cfg(test)]
mod execute_command_set_config_tests {
    use assert_fs::TempDir;
    use crate::service_configuration::ServiceType;
    use std::fs;
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn stores_the_config() {
        let temp = TempDir::new().unwrap();

        let config = TestConfiguration {
            dummy_config_path: temp.join("dummy.config")
        };

        execute_command(&config).unwrap();

        assert_eq!(
            String::from_utf8(fs::read(&config.dummy_config_path).unwrap()).unwrap(),
            String::from("my_config_value")
        )
    }

    #[test]
    fn returns_the_config_contents() {
        let temp = TempDir::new().unwrap();

        let config = TestConfiguration {
            dummy_config_path: temp.join("dummy.config")
        };

        assert_eq!(
            execute_command(&config).unwrap(),
            String::from("my_config_key: my_config_value\n")
        )
    }

    struct TestConfiguration { dummy_config_path: PathBuf }

    impl Configuration for TestConfiguration {
        fn abuse_notifications_author_name(&self) -> Option<&str> {
            None
        }

        fn abuse_notifications_from_address(&self) -> Option<&str> {
            None
        }

        fn config_file_location(&self) -> &Path {
            Path::new("/does/not/matter")
        }

        fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
            let config_file_contents = fs::read(&self.dummy_config_path).unwrap();
            let config_val = String::from_utf8(config_file_contents).unwrap();
            vec![(String::from("my_config_key"), Some(config_val))]
        }

        fn db_path(&self) -> Option<&PathBuf> {
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
            &ServiceType::Config(ConfigServiceCommands::Set)
        }

        fn store_config(&self) {
            fs::write(&self.dummy_config_path, "my_config_value").unwrap();
        }

        fn trusted_recipient(&self)-> Option<&str> {
            None
        }
    }
}

#[cfg(test)]
mod execute_command_unrecognised_service_type_tests {
    use std::path::{Path, PathBuf};
    use super::*;

    #[test]
    fn returns_error_if_not_config_service_type() {
        let config = TestConfiguration {};

        match execute_command(&config) {
            Ok(_) => panic!("Unexpected OK"),
            Err(AppError::Irrecoverable) => (),
            Err(e) => panic!("Unexpected Error({e})")
        }
    }

    struct TestConfiguration {}

    impl Configuration for TestConfiguration {
        fn abuse_notifications_author_name(&self) -> Option<&str> {
            None
        }

        fn abuse_notifications_from_address(&self) -> Option<&str> {
            None
        }

        fn config_file_location(&self) -> &Path {
            Path::new("/does/not/matter")
        }

        fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
            vec![]
        }

        fn db_path(&self) -> Option<&PathBuf> {
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

        fn store_config(&self) {
        }

        fn trusted_recipient(&self)-> Option<&str> {
            None
        }
    }
}

#[cfg(test)]
mod present_config_file_entries_tests {
    use super::*;

    #[test]
    fn returns_sorted_string_of_key_value_pairs() {
        let entries = vec![
            (String::from("biz"), Some(String::from("boz"))),
            (String::from("foo"), Some(String::from("bar"))),
            (String::from("baz"), Some(String::from("buzz"))),
        ];

        let expected = "\
                        baz: buzz\n\
                        biz: boz\n\
                        foo: bar\n\
                        ";

        assert_eq!(present_config_file_entries(entries), expected);
    }

    #[test]
    fn returns_only_config_key_if_no_value() {
        let entries = vec![
            (String::from("biz"), Some(String::from("boz"))),
            (String::from("foo"), Some(String::from("bar"))),
            (String::from("bee"), None),
            (String::from("baz"), Some(String::from("buzz"))),
        ];

        let expected = "\
                        baz: buzz\n\
                        bee: \n\
                        biz: boz\n\
                        foo: bar\n\
                        ";

        assert_eq!(present_config_file_entries(entries), expected);
    }
}
