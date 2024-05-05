use crate::errors::AppError;
use crate::service_configuration::{Configuration, ServiceType};

mod config;
mod process_message;

pub async fn execute_command<T>(config: &T) -> Result<String, AppError>
    where T: Configuration {
    match config.service_type() {
        ServiceType::ProcessMessage => process_message::execute_command(config).await,
        ServiceType::Config(_) => config::execute_command(config)
    }
}

#[cfg(test)]
mod service_execute_command_process_message_tests {
    use assert_fs::fixture::TempDir;
    use crate::cli::{ProcessArgs, SingleCli, SingleCliCommands};
    use crate::mountebank::{clear_all_impostors, setup_bootstrap_server};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::Path;

    use super::*;

    #[test]
    fn calls_service_process_message_and_returns_ok_response() {
        clear_all_impostors();
        setup_bootstrap_server();

        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("pp.sqlite3");
        let input = single_source_input();

        let config = build_config(Some(&input), None, &db_path);

        let output = tokio_test::block_on(execute_command(&config)).unwrap();

        assert!(output.contains("Abuse Email Address"));
    }

    #[test]
    fn calls_service_process_message_and_returns_err_response() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("un/ob/tai/nium");

        let config = ServiceConfiguration::new(
            Some("message_source"),
            &cli(None),
            env_var_iterator(
                Some(db_path.to_str().unwrap()),
                Some("foo.com"),
                Some("http://localhost:4545")
            )
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

    fn single_source_input() -> String {
        entry_1()
    }

    fn entry_1() -> String {
        format!(
            "From 123@xxx Sun Jun 11 20:53:34 +0000 2023\r\n{}",
            mail_body_1()
        )
    }

    fn mail_body_1() -> String {
        format!(
            "{}\r\n{}",
            "Delivered-To: victim1@test.zzz",
            "Subject: Dodgy Subject 1"
        )
    }

    fn build_config<'a>(
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
                Some("http://localhost:4545")),
        ).unwrap()
    }

    fn env_var_iterator(
        db_path_option: Option<&str>,
        trusted_recipient_option: Option<&str>,
        rdap_bootstrap_host_option: Option<&str>,
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

    fn cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs{
                reprocess_run
            })
        }
    }
}

#[cfg(test)]
mod service_execute_command_config_tests {
    use crate::cli::{ConfigArgs, ConfigCommands, SingleCli, SingleCliCommands};
    use crate::service_configuration::ServiceConfiguration;

    use super::*;

    #[test]
    fn calls_service_config_and_returns_ok_response() {
        let config = build_config();

        let expected = String::from(
            confy::get_configuration_file_path("phisher_eagle", None)
            .unwrap()
            .to_str()
            .unwrap()
        );
        let output = tokio_test::block_on(execute_command(&config)).unwrap();

        assert_eq!(output, expected);
    }

    fn build_config<'a>() -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            None,
            &cli(),
            env_var_iterator()
        ).unwrap()
    }

    fn env_var_iterator() -> Box<dyn Iterator<Item = (String, String)>>
    {
        let v: Vec<(String, String)> = vec![];

        Box::new(v.into_iter())
    }

    fn cli() -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Config(ConfigArgs{
                command: ConfigCommands::Location,
            })
        }
    }
}
