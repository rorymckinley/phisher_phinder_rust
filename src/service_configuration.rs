use crate::cli::{ConfigArgs, ConfigCommands, SingleCli, SingleCliCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Default, Deserialize, Serialize)]
struct FileConfig {
    abuse_notifications_author_name: Option<String>,
    abuse_notifications_from_address: Option<String>,
    db_path: Option<String>,
    smtp_host_uri: Option<String>,
    smtp_password: Option<String>,
    smtp_username: Option<String>,
}

pub trait Configuration {
    fn abuse_notifications_author_name(&self) -> Option<&str>;

    fn abuse_notifications_from_address(&self) -> Option<&str>;

    fn config_file_entries(&self) -> Vec<(String, Option<String>)>;

    fn config_file_location(&self) -> &Path;

    fn db_path(&self) -> Option<&PathBuf>;

    fn message_sources(&self) -> Option<&str>;

    fn rdap_bootstrap_host(&self) -> Option<&str>;

    fn reprocess_run_id(&self) -> Option<i64>;

    fn service_type(&self) -> &ServiceType;

    fn trusted_recipient(&self)-> Option<&str>;
}

#[derive(Debug, Error, PartialEq)]
pub enum ServiceConfigurationError {
    #[error("{0} is a required ENV variable")]
    MissingEnvVar(String),
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
    abuse_notifications_author_name: Option<String>,
    abuse_notifications_from_address: Option<String>,
    config_file_location: PathBuf,
    db_path: Option<PathBuf>,
    message_source: Option<&'a str>,
    rdap_bootstrap_host: Option<String>,
    reprocess_run_id: Option<i64>,
    trusted_recipient: Option<String>,
    service_type: ServiceType,
}

impl<'a> ServiceConfiguration<'a> {
    pub fn new<I>(
        message_source: Option<&'a str>,
        cli_parameters: &SingleCli,
        env_vars_iterator: I
    ) -> Result<Self, ServiceConfigurationError>
    where I: Iterator<Item = (String, String)>
    {
        // TODO Need a way to test error handling
        // TODO a panic is probably ok here given that we are dead in the water if
        // we can't lookup the config, but would expect() be  better choice in terms
        // of visibility?
        let config_file_location =
            confy::get_configuration_file_path("phisher_eagle", None).unwrap();

        match  cli_parameters {
            SingleCli {command: SingleCliCommands::Process(args), ..} => {
                if message_source.is_none() && args.reprocess_run.is_none() {
                    return Err(ServiceConfigurationError::NoMessageSource);
                }

                let env_vars: HashMap<String, String> = env_vars_iterator.collect();

                Ok(
                    Self {
                        abuse_notifications_author_name:
                            Self::extract_optional_env_var(
                                &env_vars,
                                "PP_ABUSE_NOTIFICATIONS_AUTHOR_NAME"
                            ),
                        abuse_notifications_from_address:
                            Self::extract_optional_env_var(
                                &env_vars,
                                "PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS"
                            ),
                        config_file_location,
                        db_path: Some(
                            Path::new(
                                &Self::extract_required_env_var(&env_vars, "PP_DB_PATH")?
                            ).to_path_buf(),
                        ),
                        message_source,
                        rdap_bootstrap_host:
                            Self::extract_optional_env_var(
                                &env_vars,
                                "RDAP_BOOTSTRAP_HOST"
                            ),
                        reprocess_run_id: args.reprocess_run,
                        service_type: ServiceType::ProcessMessage,
                        trusted_recipient: Some(
                            Self::extract_required_env_var(
                                &env_vars,
                                "PP_TRUSTED_RECIPIENT"
                            )?,
                        )
                    }
                )
            },
            SingleCli {command: SingleCliCommands::Config(ConfigArgs {command}), ..} => {
                Ok(
                    Self {
                        abuse_notifications_author_name: None,
                        abuse_notifications_from_address: None,
                        config_file_location,
                        db_path: None,
                        message_source: None,
                        rdap_bootstrap_host: None,
                        reprocess_run_id: None,
                        service_type: Self::determine_config_service_type(command),
                        trusted_recipient: None
                    }
                )
            }
        }
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

    fn determine_config_service_type(subcommand: &ConfigCommands) -> ServiceType {
        match subcommand {
            ConfigCommands::Location => ServiceType::Config(ConfigServiceCommands::Location),
            ConfigCommands::Set => ServiceType::Config(ConfigServiceCommands::Set),
            ConfigCommands::Show => ServiceType::Config(ConfigServiceCommands::Show),
        }
    }
}

impl<'a> Configuration for ServiceConfiguration<'a> {
    fn abuse_notifications_author_name(&self) -> Option<&str> {
        self.abuse_notifications_author_name.as_deref()
    }

    fn abuse_notifications_from_address(&self) -> Option<&str> {
        self.abuse_notifications_from_address.as_deref()
    }

    fn config_file_entries(&self) -> Vec<(String, Option<String>)> {
        let file_config: FileConfig = confy::load_path(&self.config_file_location).unwrap();

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
        ]
    }

    fn config_file_location(&self) -> &Path {
        &self.config_file_location
    }

    fn db_path(&self) -> Option<&PathBuf> {
        self.db_path.as_ref()
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

    fn service_type(&self) -> &ServiceType {
        &self.service_type
    }

    fn trusted_recipient(&self) -> Option<&str> {
        self.trusted_recipient.as_deref()
    }
}

#[cfg(test)]
mod service_configuration_process_tests {
    use crate::cli::{ProcessArgs, SingleCliCommands};
    use super::*;

    #[test]
    fn initialises_an_instance() {
        let config_file_location =
            confy::get_configuration_file_path("phisher_eagle", None)
            .unwrap();

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        let expected = ServiceConfiguration {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            config_file_location,
            db_path: Some("/path/to/db".into()),
            message_source: Some("message source"),
            rdap_bootstrap_host: None,
            reprocess_run_id: Some(99),
            service_type: ServiceType::ProcessMessage,
            trusted_recipient: Some("foo.com".into())
        };

        assert_eq!(expected, config);
    }

    #[test]
    fn returns_service_type() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(&ServiceType::ProcessMessage, config.service_type());
    }

    #[test]
    fn returns_err_if_no_db_path_env_var() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: None,
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
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
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some(""),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
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
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: None,
                }
            )
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
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some(""),
                }
            )
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
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
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
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        );

        assert!(result.is_ok());
    }

    #[test]
    fn returns_config_if_message_source_but_no_reprocess_run_id() {
        let result = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        );

        assert!(result.is_ok());
    }

    #[test]
    fn returns_db_path() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some(&Path::new("/path/to/db").to_path_buf()), config.db_path());
    }

    #[test]
    fn returns_message_sources() {
        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(None),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some("message source"), config.message_sources());
    }

    #[test]
    fn returns_reprocess_run_id() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some(999), config.reprocess_run_id());
    }

    #[test]
    fn returns_trusted_recipient() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some("foo.com"), config.trusted_recipient());
    }

    #[test]
    fn returns_rdap_bootstrap_host() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some("localhost:4545"),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some("localhost:4545"), config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_none_if_rdap_host_empty_string() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_none_if_no_rdap_host() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_abuse_notifications_from_address() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: Some("report@phishereagle.com"),
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some("report@phishereagle.com"), config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_none_if_no_abuse_notifications_from_address() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_none_if_empty_string_abuse_notifications_from_address() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: Some(""),
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_abuse_notifications_author_name() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: Some("Jo Bloggs"),
                    abuse_notifications_from_address_option: Some("report@phishereagle.com"),
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(Some("Jo Bloggs"), config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_none_if_no_abuse_notifications_author_name() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: Some("report@phishereagle.com"),
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_none_if_empty_string_abuse_notifications_author_name() {
        let config = ServiceConfiguration::new(
            None,
            &cli(Some(999)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: Some(""),
                    abuse_notifications_from_address_option: Some("report@phishereagle.com"),
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: Some(""),
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_config_file_path() {
        let config_file_location =
            confy::get_configuration_file_path("phisher_eagle", None)
            .unwrap();

        let config = ServiceConfiguration::new(
            Some("message source"),
            &cli(Some(99)),
            env_var_iterator(
                EnvVarConfig {
                    abuse_notifications_author_name_option: None,
                    abuse_notifications_from_address_option: None,
                    db_path_option: Some("/path/to/db"),
                    rdap_bootstrap_host_option: None,
                    trusted_recipient_option: Some("foo.com"),
                }
            )
        ).unwrap();

        assert_eq!(&config_file_location, config.config_file_location());
    }

    #[test]
    fn returns_collection_of_config_file_entries() {
    }

    struct EnvVarConfig<'a> {
        pub abuse_notifications_author_name_option: Option<&'a str>,
        pub abuse_notifications_from_address_option: Option<&'a str>,
        pub db_path_option: Option<&'a str>,
        pub rdap_bootstrap_host_option: Option<&'a str>,
        pub trusted_recipient_option: Option<&'a str>,
    }

    fn env_var_iterator(config: EnvVarConfig) -> Box<dyn Iterator<Item = (String, String)>> {
        let mut v: Vec<(String, String)> = vec![];

        if let Some(abuse_notifications_author_name) =
            config.abuse_notifications_author_name_option {
            v.push((
                "PP_ABUSE_NOTIFICATIONS_AUTHOR_NAME".into(), abuse_notifications_author_name.into()
            ));
        }

        if let Some(abuse_notifications_from_address) =
            config.abuse_notifications_from_address_option {
            v.push((
                "PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS".into(),
                abuse_notifications_from_address.into()
            ));
        }

        if let Some(db_path) = config.db_path_option {
            v.push(("PP_DB_PATH".into(), db_path.into()));
        }

        if let Some(rdap_bootstrap_host) = config.rdap_bootstrap_host_option {
            v.push(("RDAP_BOOTSTRAP_HOST".into(), rdap_bootstrap_host.into()))
        }

        if let Some(trusted_recipient) = config.trusted_recipient_option {
            v.push(("PP_TRUSTED_RECIPIENT".into(), trusted_recipient.into()))
        }

        Box::new(v.into_iter())
    }

    fn cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs {
                reprocess_run,
            })
        }
    }
}

#[cfg(test)]
mod service_configuration_command_tests {
    use crate::cli::{ConfigArgs, ConfigCommands, SingleCliCommands};
    use super::*;

    #[test]
    fn initialises_an_instance() {
        let config_file_location =
            confy::get_configuration_file_path("phisher_eagle", None)
            .unwrap();

        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        let expected = ServiceConfiguration {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            config_file_location,
            db_path: None,
            message_source: None,
            rdap_bootstrap_host: None,
            reprocess_run_id: None,
            service_type: ServiceType::Config(ConfigServiceCommands::Location),
            trusted_recipient: None
        };

        assert_eq!(expected, config);
    }

    #[test]
    fn returns_service_type() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(
            &ServiceType::Config(ConfigServiceCommands::Location),
            config.service_type()
        );
    }

    #[test]
    fn returns_db_path() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.db_path());
    }

    #[test]
    fn returns_message_sources() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.message_sources());
    }

    #[test]
    fn returns_reprocess_run_id() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.reprocess_run_id());
    }

    #[test]
    fn returns_trusted_recipient() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.trusted_recipient());
    }

    #[test]
    fn returns_rdap_bootstrap_host() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.rdap_bootstrap_host());
    }

    #[test]
    fn returns_abuse_notifications_from_address() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_from_address());
    }

    #[test]
    fn returns_abuse_notifications_author_name() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(None, config.abuse_notifications_author_name())
    }

    #[test]
    fn returns_config_file_path() {
        let config_file_location =
            confy::get_configuration_file_path("phisher_eagle", None)
            .unwrap();

        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();

        assert_eq!(&config_file_location, config.config_file_location());
    }

    #[test]
    fn sets_the_config_service_type_based_on_config_subcommand() {
        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Location), vec![].into_iter()
        ).unwrap();
        assert_eq!(&ServiceType::Config(ConfigServiceCommands::Location), config.service_type());

        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Set), vec![].into_iter()
        ).unwrap();
        assert_eq!(&ServiceType::Config(ConfigServiceCommands::Set), config.service_type());

        let config = ServiceConfiguration::new(
            None, &cli(ConfigCommands::Show), vec![].into_iter()
        ).unwrap();
        assert_eq!(&ServiceType::Config(ConfigServiceCommands::Show), config.service_type());
    }

    fn cli(subcommand: ConfigCommands) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Config(ConfigArgs {
                command: subcommand,
            })
        }
    }
}

#[cfg(test)]
mod config_file_entries_tests {
    use assert_fs::TempDir;
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
            entry("smtp_host_uri", "smtp.unobtainium.zzz"),
            entry("smtp_password", "smtp_pass"),
            entry("smtp_username", "smtp_user"),
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
            none_entry("smtp_host_uri"),
            none_entry("smtp_password"),
            none_entry("smtp_username"),
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
            none_entry("smtp_host_uri"),
            none_entry("smtp_password"),
            none_entry("smtp_username"),
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
        smtp_host_uri: Option<&'a str>,
        smtp_password: Option<&'a str>,
        smtp_username: Option<&'a str>,
    }

    fn store_file_config(path: &Path) {
        let config = TestConfig {
            abuse_notifications_author_name: Some("Fred Flintstone"),
            abuse_notifications_from_address: Some("fred@yabba.dabba.doo"),
            db_path: Some("/path/to/db"),
            smtp_host_uri: Some("smtp.unobtainium.zzz"),
            smtp_password: Some("smtp_pass"),
            smtp_username: Some("smtp_user"),
        };

        confy::store_path(path, config).unwrap();
    }

    fn store_file_config_sans_values(path: &Path) {
        let config = TestConfig {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
        };

        confy::store_path(path, config).unwrap();
    }

    fn service_config(config_file_location: &Path) -> ServiceConfiguration {
        ServiceConfiguration {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            config_file_location: config_file_location.into(),
            db_path: None,
            message_source: None,
            rdap_bootstrap_host: None,
            reprocess_run_id: None,
            trusted_recipient: None,
            service_type: ServiceType::ProcessMessage,
        }
    }
}
