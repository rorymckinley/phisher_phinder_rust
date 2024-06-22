use crate::errors::AppError;
use crate::service_configuration::{Configuration, ServiceType};

mod config;
mod process_message;

pub async fn execute_command<T>(config: &mut T) -> Result<String, AppError>
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
    use crate::service_configuration::{FileConfig, ServiceConfiguration};
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn calls_service_process_message_and_returns_ok_response() {
        clear_all_impostors();
        setup_bootstrap_server();

        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("pp.sqlite3");
        let input = single_source_input();

        build_config_file(&config_file_location, &db_path);

        let mut config = build_config(Some(&input), &cli, &config_file_location);

        let output = tokio_test::block_on(execute_command(&mut config)).unwrap();

        assert!(output.contains("Abuse Email Address"));
    }

    #[test]
    fn calls_service_process_message_and_returns_err_response() {
        let cli = build_cli(None);

        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let db_path = temp.path().join("un/ob/tai/nium");

        build_config_file(&config_file_location, &db_path);

        let mut config = ServiceConfiguration::new(
            Some("message_source"),
            &cli,
            &config_file_location,
        ).unwrap();

        let result = tokio_test::block_on(execute_command(&mut config));

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

    fn build_config<'a>(
        message: Option<&'a str>,
        cli: &'a SingleCli,
        config_file_location: &'a Path,
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            message,
            cli,
            config_file_location,
        ).unwrap()
    }

    fn build_cli(reprocess_run: Option<i64>) -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Process(ProcessArgs{
                reprocess_run,
                send_abuse_notifications: false,
            })
        }
    }
}

#[cfg(test)]
mod service_execute_command_config_tests {
    use assert_fs::TempDir;
    use crate::cli::{ConfigArgs, ConfigCommands, SingleCli, SingleCliCommands};
    use crate::service_configuration::ServiceConfiguration;
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn calls_service_config_and_returns_ok_response() {
        let temp = TempDir::new().unwrap();
        let config_file_location = build_config_location(&temp);
        let cli = build_cli();

        let mut config = build_config(&cli, &config_file_location);

        let output = tokio_test::block_on(execute_command(&mut config)).unwrap();

        assert_eq!(output, String::from(config_file_location.to_str().unwrap()));
    }

    pub fn build_config_location(temp: &TempDir) -> PathBuf {
        temp.path().join("phisher_eagle.conf")
    }

    fn build_config<'a>(
        cli: &'a SingleCli,
        config_file_location: &'a Path
    ) -> ServiceConfiguration<'a> {
        ServiceConfiguration::new(
            None,
            cli,
            config_file_location,
        ).unwrap()
    }

    fn build_cli() -> SingleCli {
        SingleCli {
            command: SingleCliCommands::Config(ConfigArgs{
                command: ConfigCommands::Location,
            })
        }
    }
}
