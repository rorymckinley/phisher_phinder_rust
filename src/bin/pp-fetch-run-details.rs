use clap::Parser;
use phisher_phinder_rust::cli::FetchRunDetailsCli;
use phisher_phinder_rust::persistence::{connect, find_run};
use phisher_phinder_rust::ui::{
    display_authentication_results,
    display_reportable_entities,
    display_sender_addresses_extended,
};
use std::path::Path;

fn main() {
    let cli = FetchRunDetailsCli::parse();
    let db_path = std::env::var("PP_DB_PATH").unwrap();
    let conn = connect(Path::new(&db_path)).unwrap();

    let run = find_run(&conn, cli.run_id).unwrap();

    if cli.email_addresses {
        println!(
            "{}",
            display_sender_addresses_extended(&run.data.parsed_mail.email_addresses).unwrap()
        );
    }

    if cli.authentication_results {
        println!("{}", display_authentication_results(&run.data).unwrap());
    }

    if cli.reportable_entities {
        println!("{}", display_reportable_entities(&run).unwrap());
    }

    if cli.message_source {
        println!("{}", run.message_source.data);
    }

    if cli.pipe_message_source {
        print!("{}", run.message_source.data);
    }
}
