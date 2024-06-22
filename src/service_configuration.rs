use crate::cli::{ConfigArgs, ConfigCommands, SetConfigArgs, SingleCli, SingleCliCommands};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct FileConfig {
    pub abuse_notifications_author_name: Option<String>,
    pub abuse_notifications_from_address: Option<String>,
    pub db_path: Option<String>,
    pub rdap_bootstrap_host: Option<String>,
    pub smtp_host_uri: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_username: Option<String>,
    pub trusted_recipient: Option<String>,
}

pub trait Configuration {
    fn abuse_notifications_author_name(&self) -> Option<&str>;

    fn abuse_notifications_from_address(&self) -> Option<&str>;

    fn config_file_entries(&self) -> Vec<(String, Option<String>)>;

    fn config_file_location(&self) -> &Path;

    fn db_path(&self) -> Option<&Path>;

    fn message_sources(&self) -> Option<&str>;

    fn rdap_bootstrap_host(&self) -> Option<&str>;

    fn reprocess_run_id(&self) -> Option<i64>;

    fn send_abuse_notifications(&self) -> bool;

    fn service_type(&self) -> &ServiceType;

    fn store_config(&mut self);

    fn trusted_recipient(&self)-> Option<&str>;
}

#[derive(Debug, Error, PartialEq)]
pub enum ServiceConfigurationError {
    #[error("{0} is a required ENV variable")]
    MissingEnvVar(String),
    #[error("Error reading configuration file")]
    ConfigFileReadError,
    #[error("Please pass message source to STDIN or reprocess a run.")]
    NoMessageSource,
    #[error("Fallthrough")]
    FallthroughError
}

#[derive(Debug, PartialEq)]
pub enum ServiceType {
    Config(ConfigServiceCommands),
    ProcessMessage,
}

#[derive(Debug, PartialEq)]
pub enum ConfigServiceCommands {
    Location,
    Set,
    Show,
}

#[derive(Debug, PartialEq)]
pub struct ServiceConfiguration<'a> {
    config_file: FileConfig,
    config_file_location: &'a Path,
    message_source: Option<&'a str>,
    reprocess_run_id: Option<i64>,
    send_abuse_notifications: bool,
    service_type: ServiceType,
    set_config_args: Option<&'a SetConfigArgs>,
}

impl<'a> ServiceConfiguration<'a> {
    pub fn new(
        message_source: Option<&'a str>,
        cli_parameters: &'a SingleCli,
        config_file_location: &'a Path,
    ) -> Result<Self, ServiceConfigurationError> {
        if let Ok(config_file) = confy::load_path::<FileConfig>(config_file_location) {
            let service_type = determine_config_service_type(cli_parameters);

            match  cli_parameters {
                SingleCli {command: SingleCliCommands::Process(args), ..} => {
                    Ok(
                        Self {
                            set_config_args: None,
                            config_file,
                            config_file_location,
                            message_source,
                            reprocess_run_id: args.reprocess_run,
                            send_abuse_notifications: args.send_abuse_notifications,
                            service_type,
                        }
                    )
                },
                SingleCli {command: SingleCliCommands::Config(ConfigArgs {command}), ..} => {
                    let set_config_args = match command {
                        ConfigCommands::Set(args) => Some(args),
                        _ => None,
                    };

                    Ok(
                        Self {
                            set_config_args,
                            config_file,
                            config_file_location,
                            message_source: None,
                            reprocess_run_id: None,
                            send_abuse_notifications: false,
                            service_type,
                        }
                    )
                }
            }
        } else {
            Err(ServiceConfigurationError::ConfigFileReadError)
        }
    }
}

impl<'a> Configuration for ServiceConfiguration<'a> {
    fn abuse_notifications_author_name(&self) -> Option<&str> {
        self.config_file.abuse_notifications_author_name.as_deref().filter(|v| !v.is_empty())
    }

    fn abuse_notifications_from_address(&self) -> Option<&str> {
        self.config_file.abuse_notifications_from_address.as_deref().filter(|v| !v.is_empty())
    }

    fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
        let file_config: FileConfig = confy::load_path(self.config_file_location).unwrap();

        vec![
            (
                "abuse_notifications_author_name".into(),
                file_config.abuse_notifications_author_name
            ),
            (
                "abuse_notifications_from_address".into(),
                file_config.abuse_notifications_from_address
            ),
            (
                "db_path".into(),
                file_config.db_path
            ),
            (
                "rdap_bootstrap_host".into(),
                file_config.rdap_bootstrap_host
            ),
            (
                "smtp_host_uri".into(),
                file_config.smtp_host_uri
            ),
            (
                "smtp_password".into(),
                file_config.smtp_password
            ),
            (
                "smtp_username".into(),
                file_config.smtp_username
            ),
            (
                "trusted_recipient".into(),
                file_config.trusted_recipient
            )
        ]
    }

    fn config_file_location(&self) -> &Path {
        self.config_file_location
    }

    fn db_path(&self) -> Option<&Path> {
        self.config_file.db_path.as_ref().map(|path_string| {
            Path::new(path_string)
        })
    }

    fn message_sources(&self) -> Option<&'a str> {
        self.message_source
    }

    fn rdap_bootstrap_host(&self) -> Option<&str> {
        self.config_file.rdap_bootstrap_host.as_deref().filter(|v| !v.is_empty())
    }

    fn reprocess_run_id(&self) -> Option<i64> {
        self.reprocess_run_id
    }

    fn send_abuse_notifications(&self) -> bool {
        self.send_abuse_notifications
    }

    fn service_type(&self) -> &ServiceType {
        &self.service_type
    }

    // TODO Return a Result<()>
    fn store_config(&mut self) {
        if let Some(new_config) = configuration_file_contents::new_contents(
            &self.config_file,
            self.set_config_args
        ) {
            // TODO Need a way to test error handling
            confy::store_path(self.config_file_location, &new_config).unwrap();
            self.config_file = new_config;
        }
    }

    fn trusted_recipient(&self) -> Option<&str> {
        self.config_file.trusted_recipient.as_deref()
    }
}

fn determine_config_service_type(subcommand: &SingleCli) -> ServiceType {
    match subcommand {
        SingleCli {command: SingleCliCommands::Process(_)} => ServiceType::ProcessMessage,
        SingleCli {command: SingleCliCommands::Config(ConfigArgs {command})} => {
            match command {
                ConfigCommands::Location => ServiceType::Config(ConfigServiceCommands::Location),
                ConfigCommands::Set(_) => ServiceType::Config(ConfigServiceCommands::Set),
                ConfigCommands::Show => ServiceType::Config(ConfigServiceCommands::Show),
            }
        }
    }
}

#[cfg(test)]
mod service_configuration_new_tests {
    use assert_fs::TempDir;
    use crate::cli::{ProcessArgs, SingleCliCommands};
    use test_support::*;
    use super::*;

    #[test]
    fn initialises_an_instance_for_process_command() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_process_cli(Some(99), false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            &config_file_location,
        ).unwrap();

        let expected = ServiceConfiguration {
            config_file: file_config(),
            config_file_location: &config_file_location,
            message_source: Some("message source"),
            reprocess_run_id: Some(99),
            send_abuse_notifications: false,
            service_type: ServiceType::ProcessMessage,
            set_config_args: None,
        };

        assert_eq!(expected, config);
    }

    #[test]
    fn initialises_an_instance_with_send_abuse_notifications_for_process_command() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_process_cli(Some(99), true);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            &config_file_location,
        ).unwrap();

        assert!(config.send_abuse_notifications);
    }
    #[test]
    fn initialises_an_instance_for_config_command() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_config_cli(ConfigCommands::Location);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        let expected = ServiceConfiguration {
            config_file: file_config(),
            config_file_location: &config_file_location,
            message_source: None,
            reprocess_run_id: None,
            send_abuse_notifications: false,
            service_type: ServiceType::Config(ConfigServiceCommands::Location),
            set_config_args: None,
        };

        assert_eq!(expected, config);
    }

    #[test]
    fn stores_a_reference_to_the_set_config_parameters_for_set_config_command() {
        let config_file_location = Path::new("/tmp/phisher_eagle.conf");

        let cli = build_config_cli(ConfigCommands::Set(set_config_args()));
        let config = ServiceConfiguration::new(
            None,
            &cli,
            config_file_location,
        ).unwrap();
        assert_eq!(config.set_config_args, Some(&set_config_args()));

        let cli = build_config_cli(ConfigCommands::Show);
        let config = ServiceConfiguration::new(
            None,
            &cli,
            config_file_location,
        ).unwrap();
        assert!(config.set_config_args.is_none());

        let cli = build_config_cli(ConfigCommands::Location);
        let config = ServiceConfiguration::new(
            None,
            &cli,
            config_file_location,
        ).unwrap();
        assert!(config.set_config_args.is_none());
    }

    #[test]
    fn returns_error_if_the_config_file_cannot_be_read() {
        let config_file_location = Path::new("/should/not/exist");

        let cli = build_process_cli(Some(99), false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            config_file_location,
        );

        match config {
            Err(ServiceConfigurationError::ConfigFileReadError) => (),
            Err(e) => panic!("Unexpected error response {e}"),
            Ok(_) => panic!("Did not return error"),
        }
    }

    fn build_process_cli(reprocess_run: Option<i64>, send_abuse_notifications: bool) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run,
                send_abuse_notifications,
            })
        }
    }

    fn build_config_cli(subcommand: ConfigCommands) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: subcommand,
            })
        }
    }

    fn set_config_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        }
    }
}

#[cfg(test)]
mod service_configuration_configuration_trait_tests {
    use assert_fs::TempDir;
    use crate::cli::{ProcessArgs, SingleCliCommands};
    use test_support::*;
    use super::*;

    #[test]
    fn returns_service_type() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(99), false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(&ServiceType::ProcessMessage, config.service_type());
    }

    #[test]
    fn returns_db_path() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(None, false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some(Path::new("/other/path/to/db")), config.db_path());
    }

    #[test]
    fn returns_message_sources() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(None, false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some("message source"), config.message_sources());
    }

    #[test]
    fn returns_reprocess_run_id() {
        let temp = TempDir::new().unwrap();

        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some(999), config.reprocess_run_id());
    }

    #[test]
    fn returns_trusted_recipient() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some("google.com"), config.trusted_recipient());
    }

    #[test]
    fn returns_rdap_bootstrap_host() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some("localhost:4646"), config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_none_if_rdap_bootstrap_host_empty_string() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_rdap_bootstrap_empty()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_none_if_no_rdap_bootstrap_host() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_rdap_bootstrap_none()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_abuse_notifications_from_address() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some("fred@flintstone.zzz"), config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_none_if_no_abuse_notifications_from_address() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_abuse_from_address_none()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_none_if_empty_string_abuse_notifications_from_address() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_abuse_from_address_empty()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_abuse_notifications_author_name() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(temp.path(), file_config());

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(Some("Fred Flintstone"), config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_none_if_no_abuse_notifications_author_name() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_abuse_author_name_none()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_none_if_empty_string_abuse_notifications_author_name() {
        let temp = TempDir::new().unwrap();
        let config_file_location = create_config_file(
            temp.path(),
            file_config_abuse_author_name_empty()
        );

        let cli = build_cli(Some(999), false);

        let config = ServiceConfiguration::new(
            None,
            &cli,
            &config_file_location,
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_config_file_path() {
        let config_file_location = Path::new("/tmp/phisher_eagle.conf");

        let cli = build_cli(Some(99), false);

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            config_file_location,
        ).unwrap();

        assert_eq!(config_file_location, config.config_file_location());
    }

    #[test]
    fn returns_send_abuse_notifications() {
        let config_file_location = Path::new("/tmp/phisher_eagle.conf");

        let cli = build_cli(Some(99), true);
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            config_file_location,
        ).unwrap();
        assert!(config.send_abuse_notifications());

        let cli = build_cli(Some(99), false);
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli,
            config_file_location,
        ).unwrap();
        assert!(!config.send_abuse_notifications());
    }

    fn build_cli(reprocess_run: Option<i64>, send_abuse_notifications: bool) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run,
                send_abuse_notifications,
            })
        }
    }
}

#[cfg(test)]
mod service_configuration_configuration_trait_config_file_entries_tests {
    use assert_fs::TempDir;
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn returns_current_config_file_entries() {
        let temp = TempDir::new().unwrap();

        let file_path = config_file_path(temp.path());

        store_file_config(&file_path);

        let config = service_config(&file_path);

        let expected = vec![
            entry("abuse_notifications_author_name", "Fred Flintstone"),
            entry("abuse_notifications_from_address", "fred@yabba.dabba.doo"),
            entry("db_path", "/path/to/db"),
            entry("rdap_bootstrap_host", "localhost:4545"),
            entry("smtp_host_uri", "smtp.unobtainium.zzz"),
            entry("smtp_password", "smtp_pass"),
            entry("smtp_username", "smtp_user"),
            entry("trusted_recipient", "mx.google.com"),
        ];

        assert_eq!(expected, config.config_file_entries());
    }

    #[test]
    fn returns_nones_if_config_file_has_no_values() {
        let temp = TempDir::new().unwrap();

        let file_path = config_file_path(temp.path());

        store_file_config_sans_values(&file_path);

        let config = service_config(&file_path);

        let expected = vec![
            none_entry("abuse_notifications_author_name"),
            none_entry("abuse_notifications_from_address"),
            none_entry("db_path"),
            none_entry("rdap_bootstrap_host"),
            none_entry("smtp_host_uri"),
            none_entry("smtp_password"),
            none_entry("smtp_username"),
            none_entry("trusted_recipient"),
        ];

        assert_eq!(expected, config.config_file_entries());
    }

    #[test]
    fn returns_nones_if_config_file_is_absent() {
        let temp = TempDir::new().unwrap();

        let file_path = config_file_path(temp.path());

        let config = service_config(&file_path);

        let expected = vec![
            none_entry("abuse_notifications_author_name"),
            none_entry("abuse_notifications_from_address"),
            none_entry("db_path"),
            none_entry("rdap_bootstrap_host"),
            none_entry("smtp_host_uri"),
            none_entry("smtp_password"),
            none_entry("smtp_username"),
            none_entry("trusted_recipient"),
        ];

        assert_eq!(expected, config.config_file_entries());
    }

    fn config_file_path(base_path: &Path) -> PathBuf {
        let mut file_path: PathBuf = base_path.into();
        file_path.push("phisher_eagle.conf");

        file_path
    }

    fn entry(key: &str, value: &str) -> (String, Option<String>) {
        (String::from(key), Some(String::from(value)))
    }

    fn none_entry(key: &str) -> (String, Option<String>) {
        (String::from(key), None)
    }

    #[derive(Serialize)]
    struct TestConfig<'a> {
        abuse_notifications_author_name: Option<&'a str>,
        abuse_notifications_from_address: Option<&'a str>,
        db_path: Option<&'a str>,
        rdap_bootstrap_host: Option<&'a str>,
        smtp_host_uri: Option<&'a str>,
        smtp_password: Option<&'a str>,
        smtp_username: Option<&'a str>,
        trusted_recipient: Option<&'a str>,
    }

    fn store_file_config(path: &Path) {
        let config = TestConfig {
            abuse_notifications_author_name: Some("Fred Flintstone"),
            abuse_notifications_from_address: Some("fred@yabba.dabba.doo"),
            db_path: Some("/path/to/db"),
            rdap_bootstrap_host: Some("localhost:4545"),
            smtp_host_uri: Some("smtp.unobtainium.zzz"),
            smtp_password: Some("smtp_pass"),
            smtp_username: Some("smtp_user"),
            trusted_recipient: Some("mx.google.com"),
        };

        confy::store_path(path, config).unwrap();
    }

    fn store_file_config_sans_values(path: &Path) {
        let config = TestConfig {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        };

        confy::store_path(path, config).unwrap();
    }

    fn service_config(config_file_location: &Path) -> ServiceConfiguration {
        ServiceConfiguration {
            config_file: FileConfig::default(),
            config_file_location,
            message_source: None,
            reprocess_run_id: None,
            send_abuse_notifications: false,
            service_type: ServiceType::ProcessMessage,
            set_config_args: None,
        }
    }
}

#[cfg(test)]
mod service_configuration_configuration_trait_store_config_tests {
    use assert_fs::TempDir;
    use crate::cli::SetConfigArgs;
    use std::fs;
    use super::*;

    #[test]
    fn with_nonexistent_config_stores_full_config_passed_in_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_full_set_config_args());
        let mut config = config(&config_file_location, &cli);

        config.store_config();

        let config_file_contents: FileConfig = confy::load_path(&config_file_location).unwrap();

        assert_eq!(
            config_file_contents,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4545".into()),
                smtp_host_uri: Some("smtp.rubble.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("other_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_nonexistent_config_updates_config_copy_of_file_config_with_full_parameters() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_full_set_config_args());
        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4545".into()),
                smtp_host_uri: Some("smtp.rubble.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("other_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_nonexistent_config_stores_partial_config_passed_in_via_cli(){
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_partial_set_config_args());
        let mut config = config(&config_file_location, &cli);

        config.store_config();

        let config_file_contents: FileConfig = confy::load_path(&config_file_location).unwrap();

        assert_eq!(
            config_file_contents,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: None,
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: None,
                smtp_host_uri: None,
                smtp_password: Some("other_password".into()),
                smtp_username: None,
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_nonexistent_config_stores_partial_values_in_config_copy(){
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_partial_set_config_args());
        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: None,
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: None,
                smtp_host_uri: None,
                smtp_password: Some("other_password".into()),
                smtp_username: None,
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_existing_config_stores_full_config_passed_in_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_full_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        let config_file_contents: FileConfig = confy::load_path(&config_file_location).unwrap();

        assert_eq!(
            config_file_contents,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4545".into()),
                smtp_host_uri: Some("smtp.rubble.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("other_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_existing_config_stores_full_parameters_in_config_copy() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_full_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4545".into()),
                smtp_host_uri: Some("smtp.rubble.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("other_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_existing_config_stores_partial_config_passed_in_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_partial_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        let config_file_contents: FileConfig = confy::load_path(&config_file_location).unwrap();

        assert_eq!(
            config_file_contents,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4444".into()),
                smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("smtp_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_existing_config_stores_partial_parameters_in_config_copy() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_partial_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            FileConfig {
                abuse_notifications_author_name: Some("Barney Rubble".into()),
                abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
                db_path: Some("/path/to/db".into()),
                rdap_bootstrap_host: Some("localhost:4444".into()),
                smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
                smtp_password: Some("other_password".into()),
                smtp_username: Some("smtp_user".into()),
                trusted_recipient: Some("outlook.com".into()),
            }
        )
    }

    #[test]
    fn with_nonexistent_config_creates_empty_file_if_no_config_provided_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_empty_set_config_args());

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert!(fs::read_to_string(config_file_location).unwrap().is_empty());
    }

    #[test]
    fn with_nonexistent_config_retains_empty_config_if_no_config_provided_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_empty_set_config_args());

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            FileConfig {
                abuse_notifications_author_name: None,
                abuse_notifications_from_address: None,
                db_path: None,
                rdap_bootstrap_host: None,
                smtp_host_uri: None,
                smtp_password: None,
                smtp_username: None,
                trusted_recipient: None,
            }
        );
    }

    #[test]
    fn with_existing_config_leaves_file_untouched_if_no_config_provided_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_empty_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        let config_file_contents: FileConfig = confy::load_path(&config_file_location).unwrap();

        assert_eq!(
            config_file_contents,
            existing_file_config()
        )
    }

    #[test]
    fn with_existing_config_retains_original_config_copy_if_no_config_provided_via_cli() {
        let temp = TempDir::new().unwrap();
        let config_file_location = temp.path().join("phisher_eagle.conf");
        let cli = build_cli(build_empty_set_config_args());

        create_config_file(&config_file_location);

        let mut config = config(&config_file_location, &cli);

        config.store_config();

        assert_eq!(
            config.config_file,
            existing_file_config()
        )
    }

    fn build_full_set_config_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
            db_path: Some("/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4545".into()),
            smtp_host_uri: Some("smtp.rubble.zzz".into()),
            smtp_password: Some("other_password".into()),
            smtp_username: Some("other_user".into()),
            trusted_recipient: Some("outlook.com".into()),
        }
    }

    fn build_partial_set_config_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            abuse_notifications_from_address: None,
            db_path: Some("/path/to/db".into()),
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: Some("other_password".into()),
            smtp_username: None,
            trusted_recipient: Some("outlook.com".into()),
        }
    }

    fn build_empty_set_config_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        }
    }

    fn build_cli(args: SetConfigArgs) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: ConfigCommands::Set(args),
            })
        }
    }

    fn config<'a>(
        config_file_location: &'a Path,
        cli: &'a SingleCli
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            None,
            cli,
            config_file_location,
        ).unwrap()
    }

    fn create_config_file(config_file_location: &Path) {
        confy::store_path(config_file_location, existing_file_config()).unwrap();
    }

    fn existing_file_config() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4444".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }
}

mod configuration_file_contents {
    use super::*;

    pub fn new_contents(
        existing_config: &FileConfig, set_config_args: Option<&SetConfigArgs>
    ) -> Option<FileConfig> {
        match set_config_args {
            Some(set_config_args) => {
               if set_config_args.has_values()  {
                   let abuse_notifications_author_name = select_value(
                       existing_config.abuse_notifications_author_name.clone(),
                       set_config_args.abuse_notifications_author_name.clone()
                   );
                   let abuse_notifications_from_address = select_value(
                       existing_config.abuse_notifications_from_address.clone(),
                       set_config_args.abuse_notifications_from_address.clone()
                   );
                   let db_path = select_value(
                       existing_config.db_path.clone(),
                       set_config_args.db_path.clone()
                   );
                   let rdap_bootstrap_host = select_value(
                       existing_config.rdap_bootstrap_host.clone(),
                       set_config_args.rdap_bootstrap_host.clone()
                   );
                   let smtp_host_uri = select_value(
                       existing_config.smtp_host_uri.clone(),
                       set_config_args.smtp_host_uri.clone()
                   );
                   let smtp_password = select_value(
                       existing_config.smtp_password.clone(),
                       set_config_args.smtp_password.clone()
                   );
                   let smtp_username = select_value(
                       existing_config.smtp_username.clone(),
                       set_config_args.smtp_username.clone()
                   );
                   let trusted_recipient = select_value(
                       existing_config.trusted_recipient.clone(),
                       set_config_args.trusted_recipient.clone()
                   );

                   let file_config = FileConfig {
                       abuse_notifications_author_name,
                       abuse_notifications_from_address,
                       db_path,
                       rdap_bootstrap_host,
                       smtp_host_uri,
                       smtp_password,
                       smtp_username,
                       trusted_recipient,
                   };

                   Some(file_config)
               } else {
                   None
               }
            },
            None => None,
        }
    }

    pub fn select_value(current: Option<String>, new: Option<String>) -> Option<String> {
        new.or(current)
    }
}

#[cfg(test)]
mod configuration_file_contents_new_contents_tests {
    use super::*;

    #[test]
    fn returns_set_args_with_all_args_updated() {
        let set_config_args = complete_set_args();

        let new_config = configuration_file_contents::new_contents(
            &current_config(),
            Some(&set_config_args)
        ).unwrap();

        assert_eq!(new_config, completely_updated_config());
    }

    #[test]
    fn returns_set_args_with_partial_args_updated() {
        let set_config_args = partial_set_args();

        let new_config = configuration_file_contents::new_contents(
            &current_config(),
            Some(&set_config_args)
        ).unwrap();

        assert_eq!(new_config, partially_updated_config());
    }

    #[test]
    fn returns_none_if_set_args_has_no_values() {
        let set_config_args = empty_set_args();

        let new_config = configuration_file_contents::new_contents(
            &current_config(),
            Some(&set_config_args)
        );

        assert!(new_config.is_none());
    }

    #[test]
    fn returns_none_if_no_set_args() {
        let new_config = configuration_file_contents::new_contents(&current_config(), None);

        assert!(new_config.is_none());
    }

    fn current_config() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4444".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    fn complete_set_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4545".into()),
            smtp_host_uri: Some("smtp.rubble.zzz".into()),
            smtp_password: Some("other_pass".into()),
            smtp_username: Some("other_user".into()),
            trusted_recipient: Some("outlook.com".into()),
        }
    }

    fn partial_set_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            abuse_notifications_from_address: None,
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: Some("other_pass".into()),
            smtp_username: None,
            trusted_recipient: Some("outlook.com".into()),
        }
    }

    fn empty_set_args() -> SetConfigArgs {
        SetConfigArgs {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        }
    }

    fn completely_updated_config() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            abuse_notifications_from_address: Some("barney@rubble.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4545".into()),
            smtp_host_uri: Some("smtp.rubble.zzz".into()),
            smtp_password: Some("other_pass".into()),
            smtp_username: Some("other_user".into()),
            trusted_recipient: Some("outlook.com".into()),
        }
    }

    fn partially_updated_config() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Barney Rubble".into()),
            db_path: Some("/other/path/to/db".into()),
            smtp_password: Some("other_pass".into()),
            trusted_recipient: Some("outlook.com".into()),
            ..current_config()
        }
    }
}

#[cfg(test)]
mod configuration_file_contents_select_value_tests {
    use super::*;

    #[test]
    fn selects_new_value_if_it_has_value() {
        let current = Some("Fred Flintstone".into());
        let new = Some("Barney Rubble".into());

        let selected = configuration_file_contents::select_value(current, new.clone());

        assert_eq!(selected, new);
    }

    #[test]
    fn selects_old_value_if_new_value_is_none() {
        let current = Some("Fred Flintstone".into());
        let new = None;

        let selected = configuration_file_contents::select_value(current.clone(), new);

        assert_eq!(selected, current);
    }

    #[test]
    fn selects_none_if_both_values_are_none() {
        let current = None;
        let new = None;

        let selected = configuration_file_contents::select_value(current, new);

        assert_eq!(selected, None);
    }
}

#[cfg(test)]
mod determine_config_service_type_tests {
    use crate::cli::{ProcessArgs, SingleCliCommands};
    use super::*;

    #[test]
    fn returns_process_service_type() {
        let cli = SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run: None,
                send_abuse_notifications: false,
            })
        };

        let service_type = determine_config_service_type(&cli);

        assert_eq!(ServiceType::ProcessMessage, service_type);
    }

    #[test]
    fn returns_config_location_service_type() {
        let cli = SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: ConfigCommands::Location,
            })
        };

        let service_type = determine_config_service_type(&cli);

        assert_eq!(ServiceType::Config(ConfigServiceCommands::Location), service_type);
    }

    #[test]
    fn returns_config_set_service_type() {
        let cli = SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: ConfigCommands::Set(SetConfigArgs {
                    abuse_notifications_author_name: None,
                    abuse_notifications_from_address: None,
                    db_path: None,
                    rdap_bootstrap_host: None,
                    smtp_host_uri: None,
                    smtp_password: None,
                    smtp_username: None,
                    trusted_recipient: None,
                }),
            })
        };

        let service_type = determine_config_service_type(&cli);

        assert_eq!(ServiceType::Config(ConfigServiceCommands::Set), service_type);
    }

    #[test]
    fn returns_config_show_service_type() {
        let cli = SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: ConfigCommands::Show,
            })
        };

        let service_type = determine_config_service_type(&cli);

        assert_eq!(ServiceType::Config(ConfigServiceCommands::Show), service_type);
    }
}

#[cfg(test)]
mod test_support {
    use std::path::PathBuf;
    use super::*;

    pub fn create_config_file(path: &Path, config: FileConfig) -> PathBuf {
        let config_file_location = path.join("phisher_eagle.conf");

        confy::store_path(&config_file_location, config).unwrap();

        config_file_location
    }

    pub fn file_config() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4646".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_rdap_bootstrap_none() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: None,
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_rdap_bootstrap_empty() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_abuse_from_address_none() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: None,
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4646".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_abuse_from_address_empty() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("Fred Flintstone".into()),
            abuse_notifications_from_address: Some("".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4646".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_abuse_author_name_none() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: Some("fred@flintstone.zzz".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4646".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }

    pub fn file_config_abuse_author_name_empty() -> FileConfig {
        FileConfig {
            abuse_notifications_author_name: Some("".into()),
            abuse_notifications_from_address: Some("".into()),
            db_path: Some("/other/path/to/db".into()),
            rdap_bootstrap_host: Some("localhost:4646".into()),
            smtp_host_uri: Some("smtp.unobtainium.zzz".into()),
            smtp_password: Some("smtp_pass".into()),
            smtp_username: Some("smtp_user".into()),
            trusted_recipient: Some("google.com".into()),
        }
    }
}
