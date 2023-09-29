use clap::Parser;
use phisher_phinder_rust::cli::FindOtherRunsCli;
use phisher_phinder_rust::persistence::{connect, find_run, find_runs_for_message_source};
use phisher_phinder_rust::ui::display_run_ids;
use std::path::Path;

fn main() {
    let cli = FindOtherRunsCli::parse();
    let db_path = std::env::var("PP_DB_PATH").unwrap();
    let conn = connect(Path::new(&db_path)).unwrap();

    let reference_run = find_run(&conn, cli.run_id).unwrap();

    let all_runs = find_runs_for_message_source(&conn, &reference_run.message_source);

    println!("{}", display_run_ids(&all_runs))
}
