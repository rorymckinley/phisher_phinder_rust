use std::{io, process};
use std::path::Path;
use phisher_phinder_rust::persistence::{connect, persist_message_source};

fn main() {
    let mut raw_message_sources = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_message_sources) {
            break
        }
    }

    let message_sources: Vec<String> = serde_json::from_str(&raw_message_sources).unwrap();

    match &std::env::var("PP_DB_PATH") {
        Ok(db_path) => {
            if let Ok(connection) = connect(Path::new(db_path)) {
                for message_source in &message_sources {
                    persist_message_source(&connection, message_source);
                }
            } else {
                eprintln!("PP_DB_PATH ENV variable appears to be incorrect");
                process::exit(2)
            }
        },
        _ => {
            eprintln!("PP_DB_PATH ENV variable is required");
            process::exit(1)
        }
    }
}
