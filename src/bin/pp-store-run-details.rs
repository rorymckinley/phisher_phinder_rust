use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::persistence::{connect, persist_run};
use std::path::Path;
use std::{io, process};

fn main() {
    let mut raw_input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_input) {
            break;
        }
    }

    let input: OutputData = serde_json::from_str(&raw_input).unwrap();

    match &std::env::var("PP_DB_PATH") {
        Ok(db_path) => {
            if let Ok(conn) = connect(Path::new(&db_path)) {
                let run =  persist_run(&conn, &input).unwrap();

                let output = OutputData {
                    run_id: Some(run.id.into()),
                    ..input
                };

                print!("{}", serde_json::to_string(&output).unwrap());
            } else {
                eprintln!("PP_DB_PATH ENV variable appears to be incorrect");
                process::exit(2)
            }
        }
        _ => {
            eprintln!("PP_DB_PATH ENV variable is required");
            process::exit(1)
        }
    }
}
