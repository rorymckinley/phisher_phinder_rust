use phisher_phinder_rust::message_source::MessageSource;
use std::io;

fn main() {
    let mut raw_message_sources = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_message_sources) {
            break;
        }
    }

    let message_sources: Vec<MessageSource> = serde_json::from_str(&raw_message_sources).unwrap();

    for message_source in message_sources {
        println!("{}", serde_json::to_string(&message_source).unwrap());
    }
}
