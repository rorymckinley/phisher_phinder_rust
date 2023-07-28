use phisher_phinder_rust::sources::create_from_str;
use std::io;

fn main() {
    let mut message_source = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut message_source) {
            break;
        }
    }

    let sources = create_from_str(&message_source);

    print!("{}", serde_json::to_string(&sources).unwrap());
}
