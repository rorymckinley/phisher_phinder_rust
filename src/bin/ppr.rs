use clap::Parser;
use phisher_phinder_rust::cli::SingleCli;
use phisher_phinder_rust::service;
use phisher_phinder_rust::service_configuration::ServiceConfiguration;
use std::io::{IsTerminal, stdin};
use std::process::exit;

#[tokio::main]
async fn main() {
    let cli = SingleCli::parse();

    //TODO Add error handling for this unwrap
    let config_file_location = confy::get_configuration_file_path("phisher_eagle", None).unwrap();

    match ServiceConfiguration::new(
        read_from_stdin().as_deref(),
        &cli,
        &config_file_location
    ) {
        Ok(mut config) => {
            match service::execute_command(&mut config).await {
                Ok(output) => {
                    println!("{output}");
                    exit(0)
                },
                Err(e) => {
                    eprintln!("{e}");
                    exit(2);
                }
            }
        },
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    }
}

fn read_from_stdin() -> Option<String> {
    let mut input = String::new();

    if !stdin().is_terminal() {
        loop {
            if let Ok(0) = stdin().read_line(&mut input) {
                break;
            }
        }
    }

    if !input.is_empty() {
        Some(input)
    } else {
        None
    }
}
